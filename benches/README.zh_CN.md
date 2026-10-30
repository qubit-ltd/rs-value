# 基准测试

`downstream_read_bench` 测量下游 crate 在 Wire V1 中检查并编码值时使用的读取路径。
`preflight/`、`borrowed_encode/` 和 `preflight_and_encode/` 三组分别覆盖 1、32 和
4096 个字符串值。测试夹具使用足够大的预算，使 4096 项输出能够成功编码，避免把
预算拒绝误当成性能数据。

三组分别计时以下步骤：

| 组名 | 计时范围 |
| --- | --- |
| `preflight/` | 仅显式 payload preflight。 |
| `borrowed_encode/` | DTO 在计时开始前构造；计时内由 helper 执行内部 preflight 并编码借用的 DTO。 |
| `preflight_and_encode/` | 显式 preflight、借用 DTO 构造、helper 内部 preflight 和编码都在计时内。 |

`preflight_and_encode/` 与 `borrowed_encode/` 的差值还包含借用 DTO 的构造成本，不能解释为纯
preflight 成本。

运行定向基准测试：

```text
cargo bench --all-features --bench downstream_read_bench
```

Criterion 输出是本次变更的性能证据。在当前环境无法运行基准测试时，必须明确报告
不可用，不能据此宣称性能改进。
