# Dioxus Admin Workbench 开发约定

- `crates/admin-shell/core` 只保存正式定义、编译协议和扩展契约，不依赖 Dioxus。
- `crates/admin-shell/dioxus` 负责场景、菜单、页面编排和运行时壳层，不包含具体业务页面。
- 页面能力通过 Rudi 编译期注册；不得新增第二套注册表或手写 Provider key。
- `PageDefinition` 只持久化正式扩展配置，不保存 Dioxus `Element`、HTML、CSS 或渲染中间态。
- 约定文件是壳层内置页面来源；CRUD、树表和其他页面作为独立扩展 crate 发布。
- 第一方目录不使用 `az-` 前缀；`az-` 只用于 Cargo 包名。
- 新增代码注释使用中文。
- 单个源码文件原则上不超过 800 行；入口文件只负责模块声明、导出和顶层编排。
- 完整新增、编辑表单必须由明确操作打开 Dialog，关闭后卸载。
- 删除等破坏性操作必须先打开确认 Dialog。

