## 方案目标

* 修复 Fork 仓库下 CI 工作流最近的失败原因（缓存写入权限），且不影响上游仓库和其分支。

* 提供两种互斥方案：A 在 Fork 内最小化改动；B 建独立“编译仓库”进行构建。

## 根因复盘

* CI 工作流位于 `.github/workflows/ci.yml`，缓存步骤使用 `Swatinem/rust-cache@v2`（见 `.github/workflows/ci.yml:27-29`）。

* 任务权限为 `actions: read`、`contents: read`（`.github/workflows/ci.yml:16-19`），Fork/来自 Fork 的 PR 下写缓存会报权限错误，导致工作流失败。

## 方案 A：仅在 Fork 内调整 CI（不影响上游）

* 不改 release 流程；只在 Fork 仓库的默认分支上调整 CI：

  1. 让缓存在 Fork 下只读：在缓存步骤加输入参数

     * `with: save-if: ${{ !github.event.repository.fork }}`（文件：`.github/workflows/ci.yml`，紧随 `uses: Swatinem/rust-cache@v2`）。
  2. 可选：为主仓库增强缓存写入（仅你 Fork 生效，不合并到上游）：把 CI 任务 `permissions` 调为 `actions: write, contents: read`（`.github/workflows/ci.yml:16-19`）。如需完全不影响上游，可保留 `actions: read`，但那样主仓库也无法写缓存。
  3. 可选一致性：将 `actions/checkout@v3` 升级到 `@v4`（`.github/workflows/ci.yml:20`）。

* 执行与影响控制：

  * 修改仅推送到你的 Fork，不向上游开 PR，或在 PR 时排除该文件改动。

  * 上游仓库不受影响；Fork 下 CI 不再因缓存写入报错失败。

## 方案 B：新建“编译仓库”运行 CI（完全隔离上游）

* 新建一个空仓库，例如 `bitsrun-rs-build`，只用于 Actions 构建与发布。

* 在该仓库新增工作流 `build.yml`：

  * `on: push` 或 `workflow_dispatch`

  * 步骤：

    * `actions/checkout@v4`，配置 `repository: spencerwooo/bitsrun-rs` 或你的 Fork（只读拉取上游源码）；

    * `dtolnay/rust-toolchain@stable` 安装工具链；

    * `Swatinem/rust-cache@v2`，设置 `save-if: false`（该仓库与上游完全隔离，不写缓存也避免权限问题）；

    * `cargo fmt --all -- --check`、`cargo clippy`、`cargo check --all-targets`；

  * 若需要产物（仅你仓库内上传）：使用 `actions/upload-artifact` 或 `svenstaro/upload-release-action@v2`，使用你仓库的 `GITHUB_TOKEN` 或自有 token；不会触达上游。

* 优点：对上游零改动；构建环境独立、可控。

## 验证

* 方案 A：在你的 Fork 推送一次，观察缓存步骤不再因权限失败，`fmt/clippy/check` 通过。

* 方案 B：在新仓库手动触发 `workflow_dispatch`，确认能从上游拉取源码并成功构建。

## 结论

* 若你希望继续在当前 Fork 内跑 CI，优先选方案 A；改动很小且对上游无影响。

* 若你希望完全隔离上游，选方案 B；新仓库仅负责构建与产物上传。

