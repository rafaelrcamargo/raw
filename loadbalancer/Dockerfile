# From Nginx fixed version
FROM macbre/nginx-http3:1.25.2

# Web port
EXPOSE 9999
USER root

# Removing default configuration
RUN rm /etc/nginx/nginx.conf && \
  rm /etc/nginx/conf.d/* && \
  rm /var/log/nginx/error.log

# Bind error log to stdout
RUN ln -sf /dev/stdout /var/log/nginx/error.log

# Set master configuration
COPY nginx.conf /etc/nginx/

# Setup init commands
CMD ["nginx", "-g", "daemon off;"]
