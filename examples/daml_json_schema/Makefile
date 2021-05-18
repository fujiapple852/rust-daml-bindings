NAMESPACE = fujiapple
REPOSITORY = rust-musl
TAG = 1.52-v1
.PHONY: build push
build:
	docker build --network host -t ${NAMESPACE}/${REPOSITORY}:${TAG} -t ${NAMESPACE}/${REPOSITORY}:latest .
push:
	docker push ${NAMESPACE}/${REPOSITORY}:${TAG}
