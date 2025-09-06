# Usa a versão LTS mais recente do Node.js
ARG NODE_VERSION=22
FROM node:${NODE_VERSION}-trixie

# Argumentos para UID/GID
ARG PUID=1000
ARG PGID=1000

# Criar usuário www-data com UID/GID do host
RUN groupmod -g ${PGID} node \
 && usermod -u ${PUID} -g node node

# Instalar dependências do sistema
RUN apt-get update && apt-get install -y \
    git curl unzip build-essential tini \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Criar diretórios de cache do NPM com permissões corretas
RUN mkdir -p /var/www/.npm \
    && chown -R node:node /var/www/.npm

# Definir diretório de trabalho
WORKDIR /var/www/html

# Mudar para usuário www-data
USER node

# Evitar que npm use root
ENV NPM_CONFIG_PREFIX=/var/www/.npm

# ENTRYPOINT com tini
ENTRYPOINT ["tini", "--"]

# Entrypoint padrão
CMD ["sleep", "infinity"]


