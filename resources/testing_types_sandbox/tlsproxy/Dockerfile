FROM nginx
RUN ln -sf /dev/stdout /var/log/nginx/static.log
COPY nginx.conf node.conf proxy_params ssl_params /etc/nginx/