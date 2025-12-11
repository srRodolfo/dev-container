FROM httpd:2.4

ARG PUID
ARG PGID

RUN apt-get update && apt-get install -y --no-install-recommends \
    gosu \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g $PGID devgroup || true \
    && useradd -u $PUID -g $PGID -m devuser || true

RUN sed -i 's/AllowOverride None/AllowOverride All/' /usr/local/apache2/conf/httpd.conf
RUN sed -i '/#LoadModule rewrite_module/s/^#//' /usr/local/apache2/conf/httpd.conf
RUN sed -i '/#LoadModule proxy_module/s/^#//' /usr/local/apache2/conf/httpd.conf
RUN sed -i '/#LoadModule proxy_fcgi_module/s/^#//' /usr/local/apache2/conf/httpd.conf

RUN mkdir -p /usr/local/apache2/conf/vhosts

RUN sed -i '/Include conf\/extra\/httpd-vhosts.conf/d' /usr/local/apache2/conf/httpd.conf
RUN echo "IncludeOptional conf/vhosts/*.conf" >> /usr/local/apache2/conf/httpd.conf

RUN sed -i 's/User daemon/User devuser/g' /usr/local/apache2/conf/httpd.conf
RUN sed -i 's/Group daemon/Group devgroup/g' /usr/local/apache2/conf/httpd.conf

CMD ["httpd-foreground"]

