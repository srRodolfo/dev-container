# Usa a variável do .env (PHP_VERSION) definida no docker-compose.yml
ARG PHP_VERSION=8.4
FROM php:${PHP_VERSION}-fpm

# Argumentos para UID/GID
ARG PUID=1000
ARG PGID=1000

# 1. Configuração de Usuário (Camada que raramente muda)
RUN groupmod -g ${PGID} www-data \
    && usermod -u ${PUID} -g www-data www-data

# 2. OTIMIZAÇÃO DE CACHE (Bloco Lento e Estável)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libonig-dev libxml2-dev libzip-dev \
    libpng-dev libjpeg-dev libfreetype6-dev \
    unzip git curl \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    && docker-php-ext-configure gd --with-jpeg --with-freetype \
    && docker-php-ext-install gd exif pdo pdo_mysql mbstring xml zip sockets opcache \
    && pecl install xdebug && docker-php-ext-enable xdebug

# 3. Arquivos de Configuração (Copiar arquivos antes de outras alterações)
COPY ./opcache.ini /usr/local/etc/php/conf.d/opcache.ini

# 4. Configurações e Logs (Volátil)
# Ativar exibição de todos os erros (dev)
RUN echo "display_errors=On" >> /usr/local/etc/php/conf.d/99-errors.ini \
    && echo "display_startup_errors=On" >> /usr/local/etc/php/conf.d/99-errors.ini \
    && echo "error_reporting=E_ALL" >> /usr/local/etc/php/conf.d/99-errors.ini

# Limpeza: Comenta as diretivas user/group (como havíamos discutido)
RUN sed -i 's/^user = www-data/;user = www-data/g' /usr/local/etc/php-fpm.d/www.conf \
    && sed -i 's/^group = www-data/;group = www-data/g' /usr/local/etc/php-fpm.d/www.conf

# Configuração Xdebug (Volátil)
RUN echo "xdebug.mode=develop,debug,coverage" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.start_with_request=trigger" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_host=host.docker.internal" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini \
    && echo "xdebug.client_port=9003" >> /usr/local/etc/php/conf.d/docker-php-ext-xdebug.ini

# 5. Instalação e Permissões do Composer (Volátil)
# Instalar Composer globalmente
RUN curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer

# Criar diretórios de cache do Composer com permissões corretas
RUN mkdir -p /var/www/.composer \
    && chown -R www-data:www-data /var/www/

# Dar shell ao usuário www-data e definir senha para www-data e root
RUN usermod -s /bin/bash www-data \
    && echo "www-data:1234" | chpasswd \
    && echo "root:1234" | chpasswd

# 6. Finalização
# Definir diretório de trabalho
WORKDIR /var/www/html

# Mudar para usuário www-data
USER www-data

# Evitar que composer use root
ENV COMPOSER_HOME=/var/www/.composer

# Entrypoint
CMD ["php-fpm"]