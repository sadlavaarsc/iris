# Iris 交互式模式性能审计报告

## 执行摘要

交互式模式下的缩放延迟（数秒至十几秒）并非单一瓶颈所致，而是**多个架构级设计缺陷叠加**的结果。核心病因是：**所有图像变换（缩放/平移）都在 UI 主线程以同步方式全量重算，且对高频输入事件（如鼠标滚轮）完全未做节流控制。**

---

## 关键发现（按严重程度排序）

### P0 — 主循环中全量图像重采样（主因）

**位置**：`src/main.rs:128`, `src/viewer.rs:65-115`

每次用户操作（缩放/平移/窗口调整）都会触发 `viewer.create_stateful()`，该函数内部调用 `scaled_image()`，其执行流程如下：

```
原始图像 (orig_w x orig_h)
    ↓
如果 scale ≠ 1.0：全图 Lanczos3 resize → scaled_w x scaled_h
    ↓
如果存在 offset：crop_imm → view_width_px x view_height_px
    ↓
picker.new_resize_protocol() → 终端协议编码
```

**问题点**：

1. **全图 resize 而非视口裁剪**：用户缩放至 200% 时，代码先将整张原始图像 resize 到 `2*orig_w x 2*orig_h`（例如 8000x6000 像素），然后再从其中裁剪出一小块视口区域。这意味着 90% 以上的重采样计算结果被直接丢弃。
2. **Lanczos3 滤波器**：`src/viewer.rs:93` 使用 `imageops::FilterType::Lanczos3`，这是 `image` crate 提供的质量最高但计算成本最重的滤波器（每输出像素需要采样 36 个输入像素，卷积核支持 6x6）。对于大型图像的频繁交互，这是明显的过度设计。
3. **`DynamicImage::clone()` 开销**：`src/viewer.rs:75,96` 在 scale=1 时返回 `self.original_image.clone()`。`DynamicImage` 的 clone 是深拷贝，对于高分辨率图像（如 24MB 的 4000x3000 RGB8 图像）每次 clone 分配约 48MB 内存。

**量化估算**：
- 假设 4000x3000 图像，缩放至 200%
- Lanczos3 resize 到 8000x6000 ≈ 4800 万像素 × 36 次采样 ≈ **17 亿次像素运算/次操作**
- 连续滚动 10 次 ≈ **170 亿次运算**，在单线程上轻松达到数秒延迟

---

### P0 — 高频输入事件无节流（放大器）

**位置**：`src/events.rs:63-73`, `src/main.rs:120-132`

```rust
// events.rs:65-69
MouseEventKind::ScrollUp => {
    app.zoom_in();  // 滚轮每触发一次就 +25%
}
```

**问题点**：

1. **鼠标滚轮事件频率极高**：一次普通的滚轮滑动可产生 5-20 个连续事件。
2. **每个事件独立触发全链路重算**：`main.rs:126` 中 `changed || area_changed` 条件导致每次滚轮事件都触发一次 `create_stateful()`，没有事件合并、没有防抖、没有丢弃中间帧。
3. **同步阻塞**：图像处理在主线程完成，事件队列持续堆积。用户在滚轮滑动结束后，程序仍在处理队列中的历史事件，造成"操作已结束但画面还在逐帧更新"的卡顿感。

---

### P1 — 终端协议对象反复重建

**位置**：`src/viewer.rs:46`, `src/main.rs:128`

```rust
let protocol = self.picker.new_resize_protocol(scaled);
```

**问题点**：

`new_resize_protocol()` 和 `new_protocol()` 每次都会创建新的 `StatefulProtocol` / `Protocol` 实例。对于 Kitty 协议，这通常意味着：
- 将 `DynamicImage` 编码为终端传输格式（通常是 RGBA 原始数据或 PNG 压缩）
- 生成 Kitty 转义序列（base64 编码的图像数据）
- 通过 stdout 传输给终端

每次缩放/平移都重新编码并传输整张视口图像，即使视口内容仅有微小变化。Kitty 协议虽然支持增量/局部更新，但当前实现每次都重建全新 protocol，无法利用终端侧的图像缓存机制。

---

### P1 — 渲染与变换逻辑的次优耦合

**位置**：`src/viewer.rs:88-97`, `src/ui.rs:22`

**双重 resize 问题**：

1. `scaled_image()` 已经根据 `area` 的尺寸和 `scale` 计算了目标尺寸并执行了 resize。
2. `ui.rs:22` 中 `StatefulImage::default().resize(Resize::Fit(None))` 告诉 `ratatui-image` 再次对图像做 Fit 模式的 resize。

虽然 `ratatui-image` 内部可能做了优化，但这种职责分离不清导致：
- 开发者难以判断图像变换的实际执行点
- 可能产生预期外的二次采样（尤其当 `scaled_image` 输出尺寸与 `area` 不完全匹配时）

---

### P2 — 平移 offset 使用像素单位而非单元格单位

**位置**：`src/app.rs:38-41`, `src/viewer.rs:99-100`

`App::pan()` 直接修改像素 offset（`±10` 像素），但在 `scaled_image()` 中需要将这些像素 offset 与单元格尺寸（`cell_w`, `cell_h`）进行换算。更关键的是，**每次平移都会触发完整的图像 resize + crop + protocol 重建**，即使平移只是将视口移动几个像素。

理想情况下，平移操作只需调整 crop 的偏移量，而不应重新 resize 整张图像。

---

### P2 — 缺少图像变换的增量/缓存策略

**位置**：全局

当前架构没有任何形式的变换缓存：
- 没有缩略图/多级 mipmapping
- 没有上一次 resize 结果的缓存
- 没有脏区域检测（整个图像始终全量重算）
- 没有将 resize 与 crop/protocol 创建分离为独立阶段

---

## 根因链

```
用户滚动鼠标滚轮 ─┬─→ 产生 10-20 个连续 zoom_in 事件
                  │
                  ├─→ 每个事件在主线程同步触发
                  │     create_stateful(scale, offset, area)
                  │
                  ├─→ scaled_image() 每次全量执行：
                  │     • 原始图像 → Lanczos3 resize 到 2x/4x/... 尺寸
                  │     • clone() 大内存分配
                  │     • crop_imm 丢弃 90% 计算结果
                  │
                  ├─→ new_resize_protocol() 每次重建终端协议对象
                  │     • 图像重新编码 → base64 → stdout 传输
                  │
                  └─→ 阻塞主线程，事件队列堆积
                        → 用户感受到"十几秒"的延迟
```

---

## 修复优先级建议（仅分析，不执行）

| 优先级 | 修复方向 | 预期效果 |
|--------|----------|----------|
| P0 | **先裁剪后 resize**：根据视口+offset 计算需要加载的原始图像区域，只 resize 该区域 | 将 O(全图) 降为 O(视口)，10-100x 加速 |
| P0 | **输入事件节流/防抖**：对滚轮/键盘连续事件做 50-100ms 节流，只处理最后一次 | 消除高频事件的放大效应 |
| P0 | **更换滤波器**：Lanczos3 → Triangle 或 Nearest（交互时使用，静止后再用高质量） | 单次 resize 5-10x 加速 |
| P1 | **分离 resize 与 protocol 创建**：缩放时只重新 resize，平移时只重新 crop | 平移操作接近零成本 |
| P1 | **使用 ratatui-image 的 stateful 增量更新**：避免每次重建 Protocol，利用 Kitty 图像缓存 | 减少协议编码和传输开销 |
| P2 | **背景线程图像处理**：将 resize/protocol 创建移至后台线程，主线程保持响应 | 消除 UI 阻塞感 |
| P2 | **多级缓存**：缓存常用缩放级别的 resize 结果 | 反复缩放同一范围时即时响应 |

---

## 结论

当前性能问题的**直接原因是主线程同步执行全图 Lanczos3 resize**，而**根本原因是架构上缺少"视口感知"的图像处理流程**——代码始终在操作整张图像，而非用户实际能看到的视口区域。加上高频输入事件的无节制触发，使得本可接受的单次延迟被放大为不可用的交互体验。

修复路径的核心应围绕：**"先确定用户能看到什么，再只计算那些像素"**。
