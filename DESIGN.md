# Azuki-proto design sketch

## Intro
 Azuki-proto is a Layer-4 protocol based on UDP.

 ## Package structure

 +--------------------------------+
 |            UDP Header          |
 |             8 Bytes            |
 +--------------------------------+
 |VER|      SEQ      |   OPT   |  |
 | 1 |       4       |    2    |  |
 +--------------------------------+
 |           MSGLEN            |DA|
 |          8 Bytes            |TA|
 +--------------------------------+
 |             DATA               |
 |                                |
 +--------------------------------+