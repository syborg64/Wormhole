## Guide to Using Docker Images and Test Commands (Not up to date)

#### **1. Start the Infrastructure**
```bash
docker-compose up
```
- **Purpose**: Launches the `wormhole1` and `wormhole2` services in the background.
- **Expected Events**:
  - Containers `w1` and `w2` start with their volumes (`shared_mnt1`, `shared_mnt2`).
  - Services listen on ports `8081` (w1) and `8082` (w2).

---

#### **2. Create a Network Template on w1**
```bash
docker exec -it w1 ./wormhole template
```
- **Purpose**: Initializes a default network configuration in `shared_mnt1/.global_config.toml`.
- **Expected Result**:
  ```bash
  creating network "default"
  Network configuration created at /usr/src/wormhole/virtual/.global_config.toml
  ```

---

#### **3. Create a New Pod on w1**
```bash
docker exec -it w1 ./wormhole new test
```
- **Purpose**: Creates a pod named `test` in `w1`'s network.
- **Expected Events**:
  - A `test` folder is created in `shared_mnt1`.
  - The `w1` service becomes the primary network node.

---

#### **4. Inspect the w2 Container**
```bash
docker inspect w1
```
- **Purpose**: Retrieve `w1`'s internal IP for inter-container communication.
- **Key Data**:
  ```json
  "GateWay": "172.19.0.3",
  ```

---

#### **5. Connect w2 to w1's Network**
```bash
docker exec -it w2 ./wormhole new test 172.20.0.3:8081
```
- **Purpose**: Join `w2` to the `test` network hosted by `w1`.
- **Expected Result**:
  ```bash
  Pod "test" joined network via 172.20.0.3:8081
  Syncing with peer... OK
  ```

---

### Complete Workflow
```bash
# 1. Start services
docker-compose up

# 2. Configure w1 as the primary node
docker exec -it w1 ./wormhole template
docker exec -it w1 ./wormhole new test1

# 3. Configure w2 and connect it
docker inspect w1 # → Get w1’s IP and port (e.g., GateWay:172.20.0.3)
docker exec -it w2 ./wormhole new test2 172.20.0.3:8081
```
