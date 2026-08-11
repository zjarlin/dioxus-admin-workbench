# Dioxus Admin Shell

`az-dioxus-admin-shell` 是由单一 `AdminProvider` 驱动的后台工作台。它负责新建场景、新建菜单和页面、选择约定文件或扩展、加载正式定义，以及把已编译页面交给对应的 Rudi Provider 渲染。

壳层不包含业务菜单、模型或接口地址。CRUD 由 `az-dioxus-admin-extension-crud` 独立提供。

```toml
[dependencies]
az-admin-shell-core = "2026.8.13"
az-dioxus-admin-shell = "2026.8.13"
az-dioxus-admin-extension-crud = "2026.8.13"
```

消费方实现并注册一个 `AdminProvider`，然后启动 `az_dioxus_admin_shell::App`。完整示例位于仓库 `examples/web`。
