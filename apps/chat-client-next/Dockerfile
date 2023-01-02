FROM node:latest as builder

RUN npm install

RUN npm run build

FROM steebchen/nginx-spa:stable

COPY dist/ /app

EXPOSE 80

CMD ["nginx"]