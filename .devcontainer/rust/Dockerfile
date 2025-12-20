# 使用最新 Ubuntu 24.04 基础镜像
FROM ubuntu:latest

ARG LLVM_VERSION=latest
ARG RUST_VERSION=stable
ENV DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# 1. 安装基础工具、Git 和中文语言包
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl gnupg lsb-release wget aria2 locales \
    language-pack-zh-hans software-properties-common git \
    sudo less nano htop unzip tar procps net-tools build-essential \
    && echo "zh_CN.UTF-8 UTF-8" >> /etc/locale.gen \
    && locale-gen zh_CN.UTF-8

ENV LANG=zh_CN.UTF-8 \
    LC_ALL=zh_CN.UTF-8 \
    LANGUAGE=zh_CN:en_US

# 2. 动态安装最新版 LLVM 
RUN set -eux; \
    codename="$(. /etc/os-release && echo "${VERSION_CODENAME}")"; \
    # 解析 apt.llvm.org 上的最新主版本号
    if [[ "${LLVM_VERSION}" == "latest" ]]; then \
        LLVM_VERSION="$(curl -fsSL https://apt.llvm.org/ | grep -Eo "llvm-toolchain-${codename}-[0-9]+" | sed -nE "s/.*-${codename}-([0-9]+)/\\1/p" | sort -n | tail -n1)"; \
    fi; \
    wget -O llvm.sh https://apt.llvm.org/llvm.sh && chmod +x llvm.sh; \
    # 显式安装 clang, lld(极速链接器), lldb(调试器)
    ./llvm.sh "${LLVM_VERSION}" clang clangd lld lldb; \
    # 配置 update-alternatives
    for path in /usr/bin/*-"${LLVM_VERSION}"; do \
        name=$(basename "${path}" -"${LLVM_VERSION}"); \
        if [ -f "$path" ]; then update-alternatives --install "/usr/bin/${name}" "${name}" "${path}" 100; fi; \
    done; \
    update-alternatives --install /usr/bin/ld ld /usr/bin/ld.lld-${LLVM_VERSION} 100; \
    update-alternatives --install /usr/bin/lldb lldb /usr/bin/lldb-${LLVM_VERSION} 100; \
    update-alternatives --install /usr/bin/cc cc /usr/bin/clang 100; \
    update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++ 100; \
    rm -f llvm.sh && apt-get clean && rm -rf /var/lib/apt/lists/*

# 3. 安装 Rust (参考官方 Feature 逻辑)
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile default --default-toolchain ${RUST_VERSION} \
    && rustup component add clippy rustfmt \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# 4. 配置 vscode 用户和 Zsh (Powerlevel10k)
RUN useradd -m -s /usr/bin/zsh vscode && \
    echo "vscode ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers && \
    su - vscode -c 'sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended' && \
    su - vscode -c 'ZSH_CUSTOM=$HOME/.oh-my-zsh/custom; \
        git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "$ZSH_CUSTOM/themes/powerlevel10k"; \
        git clone --depth=1 https://github.com/zsh-users/zsh-autosuggestions "$ZSH_CUSTOM/plugins/zsh-autosuggestions"; \
        git clone --depth=1 https://github.com/zsh-users/zsh-syntax-highlighting "$ZSH_CUSTOM/plugins/zsh-syntax-highlighting"; \
        sed -i "s/^ZSH_THEME=.*/ZSH_THEME=\"powerlevel10k\/powerlevel10k\"/" $HOME/.zshrc; \
        sed -i "s/^plugins=(.*/plugins=(git sudo extract zsh-autosuggestions zsh-syntax-highlighting)/" $HOME/.zshrc;'

USER vscode