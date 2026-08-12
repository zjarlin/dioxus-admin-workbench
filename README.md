# Dioxus Admin Workbench

Dioxus Admin Workbench 是一个由正式定义和编译期 Provider 驱动的后台工作台库。它负责场景、菜单、页面、约定文件与页面扩展编排；消费方负责业务资源、持久化和自定义页面。

## Packages

| Package | Responsibility |
| --- | --- |
| `az-admin-shell-core` | 场景、菜单、页面、资源、命令和扩展编译协议 |
| `az-dioxus-admin-shell` | Dioxus 工作台、页面编辑器、运行时分派和唯一 `AdminProvider` |
| `az-dioxus-admin-extension-crud` | 默认资源 CRUD 扩展 |
| `az-ui-components` | Button、Dialog、DataTable、CollectionTree 等组件 |

## Install

```toml
[dependencies]
az-admin-shell-core = "2026.8.13"
az-dioxus-admin-shell = "2026.8.17"
az-dioxus-admin-extension-crud = "2026.8.17"
```

## Register the application provider

应用只注册一个 `AdminProvider`。Provider 返回正式定义和编译产物，并负责把壳层命令保存到 PostgreSQL 或其他权威数据源。

```rust,ignore
use std::sync::Arc;

use az_dioxus_admin_shell::DynAdminProvider;

#[derive(Debug)]
struct ApplicationAdminProvider;

#[rudi::Singleton(name = std::any::type_name::<ApplicationAdminProvider>())]
fn admin_provider() -> DynAdminProvider {
    Arc::new(ApplicationAdminProvider)
}
```

`ApplicationAdminProvider` 的完整实现需要提供 `load`、`execute`、`execute_resource` 和 `generate_convention_file`。可运行实现见 [`examples/web/src/main.rs`](examples/web/src/main.rs)。

Provider 的 `key()` 和 Rudi `name` 必须来自同一个具体类型。不得维护第二个手写注册 ID。

## Register a convention page

约定页面模块名由应用标识和页面标识推导。例如 `demo` 应用的 `welcome` 页面对应 `demo__welcome` 模块。

```rust
mod demo__welcome {
    use std::sync::Arc;

    use az_dioxus_admin_shell::{
        ConventionPageContext, ConventionPageProvider, DynConventionPageProvider,
    };
    use dioxus::prelude::*;

    #[derive(Debug)]
    struct Page;

    impl ConventionPageProvider for Page {
        fn key(&self) -> &'static str {
            module_path!()
        }

        fn render(&self, context: ConventionPageContext) -> Element {
            rsx! { h2 { "{context.page.title}" } }
        }
    }

    #[rudi::Singleton(name = module_path!())]
    fn page() -> DynConventionPageProvider {
        Arc::new(Page)
    }
}
```

## Enable the CRUD extension

```rust
fn main() {
    az_dioxus_admin_extension_crud::enable();
    dioxus::launch(az_dioxus_admin_shell::App);
}
```

CRUD 不依赖消费方模型类型。消费方通过 `ResourceCatalog` 暴露字段、操作和稳定资源 ID，再通过唯一 `AdminProvider` 执行列表、新增、编辑和删除请求。

## Run the example

```bash
cd examples/web
dx serve --platform web
```

示例包含一个消费方约定页面和一个启用完整增删改查的用户资源。

## Architecture rules

- `PageDefinition` 只保存 `ConventionFile` 或扩展正式配置，不保存 Dioxus `Element`、HTML、CSS 或渲染中间态。
- 页面扩展分别注册编译器和 Dioxus 渲染器，两侧使用同一个编译器生成的 qualified key。
- CRUD 是默认扩展实现，不是 shell core 的枚举分支。
- 完整表单只在 Dialog 打开期间挂载；删除必须先确认。
- 消费方可以不依赖 CRUD，并注册自己的页面扩展。
