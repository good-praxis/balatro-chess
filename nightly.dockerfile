FROM jenkins/ssh-agent:debian-jdk17

# Switch to root to install dependencies
USER root

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libasound2-dev  \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN su - jenkins -c "curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly"

RUN echo 'export PATH="/home/jenkins/.cargo/bin:$PATH"' > /etc/profile

USER root
WORKDIR /home/jenkins
