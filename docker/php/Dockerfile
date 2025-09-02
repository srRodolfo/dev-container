# Usa a variável do .env (PHP_VERSION) definida no docker-compose.yml
ARG PHP_VERSION=8.2
FROM php:${PHP_VERSION}-fpm

# Argumentos para UID/GID
ARG PUID=1000
ARG PGID=1000

# Criar usuário www-data com UID/GID do host
RUN groupmod -g ${PGID} www-data \
    && usermod -u ${PUID} -g www-data www-data

# Instalar dependências do sistema
RUN apt-get update && apt-get install -y \
    libonig-dev libxml2-dev libzip-dev \
    libpng-dev libjpeg-dev libfreetype6-dev \
    unzip git curl \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Configurar GD com suporte a JPEG e Freetype
RUN docker-php-ext-configure gd --with-jpeg --with-freetype

# Instalar extensões PHP
RUN docker-php-ext-install gd exif pdo pdo_mysql mbstring xml zip

# Ativar exibição de todos os erros (dev)
RUN echo "display_errors=On" >> /usr/local/etc/php/conf.d/99-errors.ini \
    && echo "display_startup_errors=On" >> /usr/local/etc/php/conf.d/99-errors.ini \
    && echo "error_reporting=E_ALL" >> /usr/local/etc/php/conf.d/99-errors.ini

# Instalar Xdebug
RUN pecl install xdebug && docker-php-ext-enable xdebug

# Configuração Xdebug
RUN echo "xdebug.mode=develop,debug" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.start_with_request=yes" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_host=host.docker.internal" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_port=9003" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# Instalar Composer globalmente
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Instalar Node.js (LTS) e NPM
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Criar diretórios de cache do Composer/NPM com permissões corretas
RUN mkdir -p /var/www/.composer /var/www/.npm \
    && chown -R www-data:www-data /var/www/

# Definir diretório de trabalho
WORKDIR /var/www/html

# Mudar para usuário www-data
USER www-data

# Evitar que composer/npm use root
ENV COMPOSER_HOME=/var/www/.composer
ENV NPM_CONFIG_PREFIX=/var/www/.npm

# Entrypoint
CMD ["php-fpm"]