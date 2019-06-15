NAMESPACE = fujiapple852
REPOSITORY = rust-daml
TAG = 1.36
.PHONY: build push
build:
	docker build --network host -t ${NAMESPACE}/${REPOSITORY}:${TAG} -t ${NAMESPACE}/${REPOSITORY}:latest .
push:
	docker push ${NAMESPACE}/${REPOSITORY}:${TAG}
