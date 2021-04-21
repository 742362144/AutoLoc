FROM redis
ADD libredismodule.so /etc/storageloc/

RUN mkdir -p /var/log/redis
RUN mkdir -p /var/lib/redis
RUN chmod -R 777 /var/lib/redis
RUN touch /var/log/redis/redis.log
RUN chmod 777 /var/log/redis/redis.log
RUN chmod -R 777 /tmp
COPY redis.conf /etc/redis/redis.conf

CMD [ "redis-server",  "/etc/redis/redis.conf", "--loadmodule", "/etc/storageloc/libredismodule.so"]