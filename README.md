# 小息 xiaoxi

[![Build Tauri](https://github.com/licc168/xiaoxi/actions/workflows/build-tauri.yml/badge.svg)](https://github.com/licc168/xiaoxi/actions/workflows/build-tauri.yml)

用 Tauri 做的工间休息提醒：到点弹出全屏课间操，跟着扭脖子、动动手、扩扩胸。

## 功能

- 自定义工作间隔（分钟）和休息时长（分钟）
- 快捷方案：课间 45/10、番茄 25/5、专注 50/10、短休 30/5
- 休息时全屏倒计时，跟做课间操教程（颈部、肩部、扩胸、体转、眼保健操等）
- 语音口令 + 节拍提示音
- 严格模式：休息结束前不能跳过
- 关闭主窗口会进入托盘，计时继续

## 开发

需要：Node.js、Rust。Windows 上还要 Visual Studio C++ Build Tools；macOS 上还要 Xcode Command Line Tools。

```bash
npm install
npm run tauri dev
```

打包：

```bash
npm run tauri build
```

安装包：

- Windows：`src-tauri/target/release/bundle/nsis/`
- macOS：`src-tauri/target/release/bundle/dmg/`

主窗口点「试做 45 秒」可以立刻预览全屏课间操。

## GitHub Actions 打包

推送到 `main` 或手动运行 workflow 时，会同时打 Windows 安装包（NSIS）和 macOS 安装包（Apple 芯片 + Intel 两份 DMG）。打包结果在对应 [Actions](https://github.com/licc168/xiaoxi/actions) 运行的 Artifacts 里。

打版本标签会发布到 [GitHub Releases](https://github.com/licc168/xiaoxi/releases)：

```bash
git tag v0.1.1
git push origin v0.1.1
```
