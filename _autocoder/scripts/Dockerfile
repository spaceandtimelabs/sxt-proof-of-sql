FROM autocoder:base_image

# Give all users full permissions to /root
RUN chmod -R a+rwx /root

# Install additional dependencies
RUN apt-get update && apt-get install -y \
    clang \
    lld \
    nodejs \
    npm \
    curl

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"
RUN chmod -R a+rwx /root/.cargo

# Install Foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:${PATH}"
RUN foundryup
RUN chmod -R a+rwx /root/.foundry

# Install solhint
RUN npm install -g solhint

# Set entrypoint
ENTRYPOINT ["bash"]