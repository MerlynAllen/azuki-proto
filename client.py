import socket
import time
LISTEN_PORT = 12345
LISTEN_ADDR = "0.0.0.0"

SERVER_ADDR = "la1.merlyn.eu.org"
SERVER_PORT = 60000
client = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)
client.bind((LISTEN_ADDR, LISTEN_PORT))

# init

client.sendto(f"{LISTEN_PORT}".encode(), (SERVER_ADDR, SERVER_PORT))
# server return p2p peer info
data, peer = client.recvfrom(1024)
print(data, peer)
data = data.decode()
peer_addr, peer_port = tuple(data.split(" "))
peer_port = int(peer_port)

# handshake to p2p peer
client.sendto(b"HELLO!", (peer_addr, peer_port))
print("sent")
time.sleep(2)
while 1:
    data, peer = client.recvfrom(1024)
    print(peer, data)
