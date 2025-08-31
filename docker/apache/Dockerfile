FROM httpd:2.4

# Habilitar .htaccess
RUN sed -i 's/AllowOverride None/AllowOverride All/' /usr/local/apache2/conf/httpd.conf

# Habilitar os módulos necessários para PHP-FPM
RUN sed -i '/#LoadModule rewrite_module/s/^#//' /usr/local/apache2/conf/httpd.conf
RUN sed -i '/#LoadModule proxy_module/s/^#//' /usr/local/apache2/conf/httpd.conf
RUN sed -i '/#LoadModule proxy_fcgi_module/s/^#//' /usr/local/apache2/conf/httpd.conf

# Virtual Host
COPY ./myproject.conf /usr/local/apache2/conf/extra/httpd-vhosts.conf
RUN echo "Include conf/extra/httpd-vhosts.conf" >> /usr/local/apache2/conf/httpd.conf
