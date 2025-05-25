## **Installation**
1. Install Docker. Link for Ubuntu: https://docs.docker.com/engine/install/ubuntu/
2. Run this command in the root of the project
    ```
    docker pull ghcr.io/galoisinc/crux-mir:0.10
    ```
3. To run the tests, use this command
    ```
    docker run --rm -it --mount type=bind,source=$(pwd),target=/workspace -w /workspace ghcr.io/galoisinc/crux-mir:0.10
    ```
