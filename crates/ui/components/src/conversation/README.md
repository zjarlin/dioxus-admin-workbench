# 对话工作区外观

`UiStylesheets` 加载 `style.css` 与 `codex.css`。业务侧只组合语义类名、状态和共享控件，不保存 CSS。

`main.dx-conversation[data-appearance="codex"]` 开启桌面对话外观：侧栏 260px、顶栏 46px、内容最大宽度 768px、输入区圆角 22px；通过 `data-sidebar-collapsed` 控制桌面侧栏，`data-history` 控制移动端抽屉，`data-context` 控制右侧上下文。`.dx-conversation__main[data-empty="true"]` 将欢迎语与输入框置于起始视野。消息区和侧栏各自滚动，输入框固定在工作区内。

消费者从 `__history`、`__main`（`__header`、`__messages`、`__composer-wrap`）、可选 `__context` 三个区域开始组合。`__composer-wrap` 内放表单 `__composer` 与 `__composer-footer`；模型、设备使用共享 `Select` 的 `SelectPlacement::Top`。`__message[data-role="user"]` 右侧气泡，助手消息使用普通 Markdown。已有默认对话外观不受影响。

颜色支持系统深色与显式 `html[data-theme]`；正文和控件均使用系统字体。图标、品牌和业务能力由使用方提供。

尺寸、圆角和阴影以 26.915.31945（build 9922）桌面安装包的静态样式为对照，独立实现为 Dioxus 布局；没有引入其 JavaScript、React 运行时或应用资源。没有完成原应用运行窗口的逐像素差异验收，不能以此声明像素完全相同。
