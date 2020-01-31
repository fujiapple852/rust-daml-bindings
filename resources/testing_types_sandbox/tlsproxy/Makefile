NAMESPACE = daml
REPOSITORY = sandbox-tls-proxy
CONTAINER_NAME = sandbox-tls-proxy

.PHONY: build run stop tail bash del
build: 
	docker build -t ${NAMESPACE}/${REPOSITORY}:latest .
run: 
	docker run --rm -d --name ${CONTAINER_NAME} -p 44380:44380 -p 44381:44381 -v $(shell pwd)/certs/:/certs ${NAMESPACE}/${REPOSITORY}:latest
stop:
	docker stop ${CONTAINER_NAME}
tail:
	docker logs -f ${CONTAINER_NAME}
bash:
	docker exec -it ${CONTAINER_NAME} /bin/bash
del:
	docker rm ${CONTAINER_NAME}
