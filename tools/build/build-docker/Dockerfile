FROM rust

RUN apt update

# Install Python
RUN apt install -y python3 python3-pip python3-toml python3-aiohttp

# Install flatpak
RUN apt install -y flatpak flatpak-builder
RUN flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# Install flatpak-builder-tools
COPY flatpak-builder-tools/cargo/flatpak-cargo-generator.py /flatpak-cargo-generator.py
COPY run.sh /run.sh

VOLUME ["/data"]
ENTRYPOINT ["/run.sh"]