# Dioxus Admin UI Components

`az-ui-components` 提供后台工作台使用的 Dioxus 基础控件与复合组件。基础控件来自 Dioxus Components registry 提交 `bf007c15d0cf4d04d3181cc46cf12325aa773955`，并在本 crate 内保持可直接发布的依赖边界。

## 组件

- `badge`、`button`、`checkbox`、`dialog`、`input`、`select`、`textarea`：固定 registry 源码组件。
- `data_table`：支持树形表头、固定列、合并单元格、编辑器和右侧面板的通用数据表格。
- `collection_tree`：以同一套选择、展开和行渲染契约展示扁平集合或层级树。
- `spatial`：由组件内部管理节点定位和树缩进，消费方不生成 inline style。

业务页面只提供领域数据与渲染插槽，不在本 crate 固化模型、菜单或运行时记录语义。

```toml
[dependencies]
az-ui-components = "2026.8.17"
```

`az-dioxus-admin-shell` 会自动加载组件主题、布局、工具类和控件样式。直接使用本 crate 时，
在应用根节点渲染一次 `UiStylesheets`；全部样式资产随 crate 发布，消费方不保存、复制或注入 CSS。
