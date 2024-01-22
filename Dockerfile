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
        netcat curl wget zip unzip        \
        cron sudo gosu dos2unix  jq       \
        tzdata python3 python3-pip        \
    && rm -rf /var/lib/apt/lists/*        \
    && gosu nobody true                   \
    && dos2unix

RUN addgroup --system steam     \
    && adduser --system         \
      --home /home/steam        \
      --shell /bin/bash         \
      steam                     \
    && usermod -aG steam steam  \
    && chmod ugo+rw /tmp/dumps
#COPY --chown=steam:steam /root/.steam /home/steam/.steam
#COPY --chown=steam:steam /root/.local /home/steam/.local


# Container informaiton
ARG GITHUB_SHA="not-set"
ARG GITHUB_REF="not-set"
ARG GITHUB_REPOSITORY="not-set"

ENV PUID=1000
ENV PGID=1000

RUN usermod -u ${PUID} steam                                \
    && groupmod -g ${PGID} steam                            \
    && echo "steam ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

USER steam

WORKDIR /home/steam

ENV HOME=/home/steam
ENV USER=steam
ENV LD_LIBRARY_PATH="/home/steam/.steam/sdk32:${LD_LIBRARY_PATH}"
ENV LD_LIBRARY_PATH="/home/steam/.steam/sdk64:${LD_LIBRARY_PATH}"

COPY --chown=${PUID}:${PGID} ./Pipfile ./Pipfile.lock /home/steam/scripts/

ENV PATH="/home/steam/.local/bin:${PATH}"

RUN pip3 install pipenv \
    && cd /home/steam/scripts \
    && pipenv install --system --deploy --ignore-pipfile

COPY --chown=${PUID}:${PGID} ./scripts /home/steam/scripts


EXPOSE 8211/udp
EXPOSE 27015/udp

RUN echo "source /home/steam/scripts/utils.sh" >> /home/steam/.bashrc

#HEALTHCHECK --interval=1m --timeout=3s \
#    CMD pidof valheim_server.x86_64 || exit 1

ENTRYPOINT ["/bin/bash","/home/steam/scripts/entrypoint.sh"]
