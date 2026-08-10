# Dioxus Admin CRUD Extension

`az-dioxus-admin-extension-crud` 是 `az-dioxus-admin-shell` 的默认资源增删改查页面扩展。它通过 Rudi 同时注册配置编译器和 Dioxus 渲染器。

```toml
[dependencies]
az-dioxus-admin-extension-crud = "2026.8.11"
```

消费方只需让该 crate 进入最终二进制并调用 `az_dioxus_admin_extension_crud::enable()`。Studio 会自动出现“资源增删改查”页面类型，运行时通过唯一 `AdminProvider` 执行资源请求。
