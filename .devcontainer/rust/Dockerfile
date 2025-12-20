FROM mcr.microsoft.com/devcontainers/base:ubuntu

# 允许在 devcontainer.json 中通过参数修改版本
ARG LLVM_VERSION=latest
# 避免交互式安装
ENV DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# 1. 安装基础依赖及本地化支持（包括并行下载工具）
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl gnupg lsb-release wget aria2 locales software-properties-common \
    language-pack-zh-hans language-pack-zh-hans-base zsh fonts-powerline fonts-noto-cjk \
    sudo less nano htop lsof unzip zip tar procps iproute2 net-tools git \
    && echo "zh_CN.UTF-8 UTF-8" >> /etc/locale.gen \
    && locale-gen zh_CN.UTF-8

# 在 locale 生成之后再设置中文环境
ENV LANG=zh_CN.UTF-8 \
    LC_ALL=zh_CN.UTF-8 \
    LANGUAGE=zh_CN:en_US

# 1.1 更新 Git 到最新版本 (PPA)
RUN add-apt-repository -y ppa:git-core/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends --only-upgrade git

# 2. 使用 LLVM 官方脚本安装（仅安装核心组件，避免 all 套餐过大）
RUN set -eux; \
    export LLVM_SKIP_SIGNATURE=yes; \
    codename="$(. /etc/os-release && echo "${VERSION_CODENAME}")"; \
    RESOLVED_VERSION=""; \
    \
    if [[ "${LLVM_VERSION}" == "latest" ]]; then \
        PARSED_VERSION="$( \
            curl -fsSL --max-time 10 --connect-timeout 5 -H "Accept-Encoding: gzip, deflate" https://apt.llvm.org/ 2>/dev/null \
            | zcat 2>/dev/null \
            | grep -Eo "llvm-toolchain-${codename}-[0-9]+" \
            | sed -nE "s/.*-([0-9]+)/\1/p" \
            | sort -rn | head -n1 \
        )"; \
        if [[ -n "${PARSED_VERSION}" ]]; then \
            RESOLVED_VERSION="${PARSED_VERSION}"; \
            echo "成功检测到最新 LLVM 版本: ${RESOLVED_VERSION}"; \
        else \
            echo "警告: 无法检测 ${codename} 的最新 LLVM 版本，将使用默认版本。"; \
        fi; \
    else \
        RESOLVED_VERSION="${LLVM_VERSION}"; \
    fi; \
    \
    # 使用 wget 下载 llvm.sh 脚本
    wget --timeout=20 --tries=3 -O llvm.sh https://apt.llvm.org/llvm.sh; \
    chmod +x llvm.sh; \
    \
    # 执行安装（仅安装 clang/clangd/lld/lldb，避免 all 套餐拉取整套工具链）
    if [[ -n "${RESOLVED_VERSION}" ]]; then \
        echo "安装 LLVM ${RESOLVED_VERSION}..."; \
        ./llvm.sh "${RESOLVED_VERSION}" clang clangd lld lldb; \
    else \
        echo "无版本号，不带参数执行 llvm.sh..."; \
        ./llvm.sh clang clangd lld lldb; \
    fi; \
    \
    # 配置 update-alternatives 使命令不带版本后缀
    ALT_VER=""; \
    if command -v clang >/dev/null 2>&1; then \
        ALT_VER="$(clang --version | sed -nE 's/.*clang version ([0-9]+).*/\1/p' | head -n1)"; \
    fi; \
    if [[ -z "${ALT_VER}" && -n "${RESOLVED_VERSION}" ]]; then \
        ALT_VER="${RESOLVED_VERSION}"; \
    fi; \
    if [[ -n "${ALT_VER}" ]]; then \
        for path in /usr/bin/*-"${ALT_VER}"; do \
            if [ -f "$path" ]; then \
                name=$(basename "${path}" -"${ALT_VER}"); \
                update-alternatives --install "/usr/bin/${name}" "${name}" "${path}" 100; \
            fi; \
        done; \
    fi; \
    \
    # 设置 cc 和 c++ 默认指向 clang
    update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100; \
    update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100; \
    \
    # 建立 /usr/local/bin 软链接以确保 PATH 优先命中
    for tool in clang clang++ clangd lld lldb; do \
        if command -v "${tool}" >/dev/null 2>&1; then \
            ln -sf "$(command -v "${tool}")" "/usr/local/bin/${tool}"; \
        fi; \
    done; \
    \
    # 清理临时文件和包管理缓存
    rm -f llvm.sh && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY .devcontainer/rust/p10k.zsh /tmp/p10k.zsh

# 3. 预装 oh-my-zsh、Powerlevel10k 和常用插件
RUN usermod -s /usr/bin/zsh vscode

RUN su - vscode -c '\
    export RUNZSH=no CHSH=no; \
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"; \
    ZSH_CUSTOM=${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}; \
    git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "$ZSH_CUSTOM/themes/powerlevel10k"; \
    git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions "$ZSH_CUSTOM/plugins/zsh-autosuggestions"; \
    git clone --depth=1 https://github.com/zsh-users/zsh-syntax-highlighting "$ZSH_CUSTOM/plugins/zsh-syntax-highlighting"; \
    git clone --depth=1 https://github.com/zsh-users/zsh-history-substring-search "$ZSH_CUSTOM/plugins/zsh-history-substring-search"; \
    sed -i "s/^ZSH_THEME=.*/ZSH_THEME=\"powerlevel10k\/powerlevel10k\"/" ~/.zshrc; \
    sed -i "s/^plugins=(.*/plugins=(git sudo extract zsh-autosuggestions zsh-syntax-highlighting zsh-history-substring-search)/" ~/.zshrc; \
    grep -q "p10k.zsh" ~/.zshrc || echo "[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh" >> ~/.zshrc; \
    { echo ""; echo "# 中文环境配置"; echo "export LANG=zh_CN.UTF-8"; echo "export LC_ALL=zh_CN.UTF-8"; echo "export LANGUAGE=zh_CN:en_US"; } >> ~/.zshrc \
    '

COPY --chown=vscode:vscode .devcontainer/rust/p10k.zsh /home/vscode/.p10k.zsh

# 4. 安装 Rust 工具链
ARG RUST_VERSION=stable
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile default --default-toolchain ${RUST_VERSION} && \
    rustup component add clippy rustfmt && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# 5. 添加编译目标支持
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-gnu

USER vscode

# 6. 为 vscode 用户配置 Rust 环境变量持久化
RUN echo '' >> ~/.zshrc && \
    echo '# Rust 环境变量' >> ~/.zshrc && \
    echo 'export RUSTUP_HOME=/usr/local/rustup' >> ~/.zshrc && \
    echo 'export CARGO_HOME=/usr/local/cargo' >> ~/.zshrc && \
    echo 'export PATH="$CARGO_HOME/bin:$PATH"' >> ~/.zshrc && \
    echo '' >> ~/.bashrc && \
    echo '# Rust 环境变量' >> ~/.bashrc && \
    echo 'export RUSTUP_HOME=/usr/local/rustup' >> ~/.bashrc && \
    echo 'export CARGO_HOME=/usr/local/cargo' >> ~/.bashrc && \
    echo 'export PATH="$CARGO_HOME/bin:$PATH"' >> ~/.bashrc