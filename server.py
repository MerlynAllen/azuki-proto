import socket
server = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)
server.bind(("0.0.0.0", 60000))
pair = []
while 1:
    data, peer1 = server.recvfrom(1024)
    print(peer1, data)
    data, peer2 = server.recvfrom(1024)
    print(peer2, data)
    pair.append((peer1, peer2))
    # share to clients
    server.sendto(f"{peer2[0]} {peer2[1]}".encode(), peer1)
    server.sendto(f"{peer1[0]} {peer1[1]}".encode(), peer2)
