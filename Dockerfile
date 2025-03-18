# --------------- #
# -- Steam CMD -- #
# --------------- #
FROM steamcmd/steamcmd:ubuntu

ENV TZ=America/Los_Angeles
ENV PYTHONUNBUFFERED=1
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update                        \
    && apt-get upgrade -y                 \
    && apt-get install -y -qq             \
        build-essential                   \
        htop net-tools nano gcc g++ gdb   \
        curl wget zip unzip               \
        cron sudo gosu dos2unix  jq       \
        tzdata                            \
    && rm -rf /var/lib/apt/lists/*        \
    && gosu nobody true                   \
    && dos2unix

# Remove any existing user or group with ID 1000
RUN if getent passwd 1000 > /dev/null; then userdel $(getent passwd 1000 | cut -d: -f1); fi \
    && if getent group 1000 > /dev/null; then groupdel $(getent group 1000 | cut -d: -f1); fi \
    && groupadd -g 1000 steam \
    && useradd -u 1000 -g 1000 \
      -d /home/steam \
      -s /bin/bash \
      -m steam \
    && chmod ugo+rw /tmp/dumps


# Container informaiton
ARG GITHUB_SHA="not-set"
ARG GITHUB_REF="not-set"
ARG GITHUB_REPOSITORY="not-set"

RUN echo "steam ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

USER steam

WORKDIR /home/steam

ENV HOME=/home/steam
ENV USER=steam
ENV LD_LIBRARY_PATH="/home/steam/.steam/sdk32:${LD_LIBRARY_PATH}"
ENV LD_LIBRARY_PATH="/home/steam/.steam/sdk64:${LD_LIBRARY_PATH}"

COPY --chown=${PUID}:${PGID} ./Pipfile ./Pipfile.lock /home/steam/scripts/

ENV PATH="/home/steam/.local/bin:${PATH}"

COPY --chown=${PUID}:${PGID} ./scripts/entrypoint.sh /entrypoint.sh

RUN mkdir -p $HOME/.steam \
    && mkdir -p $HOME/palworld \
    && ln -s $HOME/.local/share/Steam/steamcmd/linux32 $HOME/.steam/sdk32 \
    && ln -s $HOME/.local/share/Steam/steamcmd/linux64 $HOME/.steam/sdk64 \
    && ln -s $HOME/.steam/sdk32/steamclient.so $HOME/.steam/sdk32/steamservice.so || true \
    && ln -s $HOME/.steam/sdk64/steamclient.so $HOME/.steam/sdk64/steamservice.so || true

WORKDIR /home/steam/palworld

EXPOSE 8211/udp
EXPOSE 27015/udp

COPY --from=mbround18/gsm-reference:sha-7ec6fa9 /app/palworld /usr/local/bin/palworld

ENTRYPOINT ["/entrypoint.sh"]