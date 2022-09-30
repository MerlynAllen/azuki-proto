import socket
import time
import struct

LISTEN_PORT = 12346
LISTEN_ADDR = "::"

SERVER_ADDR = "::1"
SERVER_PORT = 12345
client = socket.socket(family=socket.AF_INET6, type=socket.SOCK_DGRAM)
client.bind((LISTEN_ADDR, LISTEN_PORT))
# data = input().encode()


class Flags:
    SYN = 1
    ACK = 2
    FIN = 4
    RST = 8
    PSH = 16
    URG = 32


class AzukiPack:
    ver = 1
    seq = 0
    opt = Flags.SYN
    data = b""

    def __init__(self, ver, seq, opt, data):
        self.ver = ver
        self.seq = seq
        self.opt = opt
        self.data = data

    def pack(self):
        return struct.pack("<BLHLQ", *[self.ver, self.seq, self.opt, len(self.data)]) + self.data

    @staticmethod
    def unpack(msg):
        (ver, seq, opt) = struct.unpack("<BLHLQ", msg[:15])


pack = AzukiPack(1, 0, Flags.SYN, b"")
client.sendto(pack.pack(), (SERVER_ADDR, SERVER_PORT))
pack.opt = Flags.ACK
client.sendto(b, (SERVER_ADDR, SERVER_PORT))
