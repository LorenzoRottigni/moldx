FROM postgres:16-alpine

ENV POSTGRES_USER=playground
ENV POSTGRES_PASSWORD=playground
ENV POSTGRES_DB=playground

COPY init.sql /docker-entrypoint-initdb.d/

EXPOSE 5432
