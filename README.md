# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Type Support For `.vue` Imports in TS

Since TypeScript cannot handle type information for `.vue` imports, they are shimmed to be a generic Vue component type by default. In most cases this is fine if you don't really care about component prop types outside of templates. However, if you wish to get actual prop types in `.vue` imports (for example to get props validation when using manual `h(...)` calls), you can enable Volar's Take Over mode by following these steps:

1. Run `Extensions: Show Built-in Extensions` from VS Code's command palette, look for `TypeScript and JavaScript Language Features`, then right click and select `Disable (Workspace)`. By default, Take Over mode will enable itself if the default TypeScript extension is disabled.
2. Reload the VS Code window by running `Developer: Reload Window` from the command palette.

You can learn more about Take Over mode [here](https://github.com/johnsoncodehk/volar/discussions/471).

## Env Settings for Android
```powershell
# for android build
$Env:PROJECT_DIR="C:\Users\capre\serial-brain-rs\src-tauri\gen\android"
$Env:LIBCLANG_PATH="C:\Users\capre\scoop\apps\llvm\current\bin"
$Env:OPENCV_LINK_LIBS="opencv_core,opencv_imgproc,opencv_imgcodecs"
$Env:OPENCV_LINK_PATHS="$PROJECT_DIR\ext\OpenCV-android-sdk\sdk\native\staticlibs\arm64-v8a,$PROJECT_DIR\ext\OpenCV-android-sdk\sdk\native\3rdparty\libs\arm64-v8a"
$Env:OPENCV_INCLUDE_PATHS="$PROJECT_DIR\ext\OpenCV-android-sdk\sdk\native\jni\include"
```

## Env Settings for Windows Dev
```powershell
# for windows build
$Env:LIBCLANG_PATH="C:\Users\capre\scoop\apps\llvm\current\bin"
$Env:OPENCV_LINK_LIBS="opencv_world4110"
$Env:OPENCV_LINK_PATHS="C:\Users\capre\Downloads\opencv\build\x64\vc16\lib"
$Env:OPENCV_INCLUDE_PATHS="C:\Users\capre\Downloads\opencv\build\include\opencv2"
```