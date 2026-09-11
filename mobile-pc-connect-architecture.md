# Mobile ↔ Desktop External Connectivity Architecture

## 1. 목표

오픈소스 애플리케이션에 모바일 앱을 추가한다.

Desktop 앱은 사용자의 로컬 기기에서 실행되며 데이터를 보유한다.
Mobile 앱은 사용자의 Desktop 앱에 원격으로 접속해야 한다.

핵심 요구사항:

- 사용자는 서비스 계정을 만들 필요가 없어야 한다.
- Tailscale 계정/설치에 의존하지 않는다.
- 중앙 서버에 사용자 데이터를 저장하지 않는다.
- Desktop은 공유기/NAT 뒤에 있을 수 있다.
- 외부 네트워크(모바일 LTE/5G 등)에서도 Desktop에 접근할 수 있어야 한다.
- 가능한 경우 Mobile ↔ Desktop 직접 연결(P2P)을 사용한다.
- P2P가 불가능한 경우에만 Relay를 사용한다.
- 첫 연결은 QR Code 기반 pairing을 목표로 한다.
- 보안은 애플리케이션 레벨에서 보장한다.

---

# 2. 권장 아키텍처

최종 목표 구조:

```text
                    ┌──────────────────────┐
                    │     Backend          │
                    │                      │
                    │  Signaling           │
                    │  Device Discovery    │
                    │  Optional Relay      │
                    └──────────┬───────────┘
                               │
                         최소 metadata
                               │
               ┌───────────────┴───────────────┐
               │                               │
               │       WebRTC / P2P            │
               │                               │
               ▼                               ▼
        ┌──────────────┐                ┌──────────────┐
        │ Mobile App   │                │ Desktop App  │
        │              │                │              │
        │ Private Key  │                │ Private Key  │
        │ Public Key   │                │ Public Key   │
        └──────────────┘                │ Local API    │
                                        │ Local Data   │
                                        └──────────────┘
````

 P2P 연결이 실패하면:

```
Mobile
   │
   │ encrypted connection
   ▼
┌──────────────┐
│ Relay Server │
└──────┬───────┘
       │
       │ encrypted connection
       ▼
   Desktop
```

 Relay는 가능하면 데이터를 해석하지 않는 dumb relay로 설계한다.

---

 # 3\. 연결 방식 우선순위

 다음 순서로 연결을 시도한다.

```
1. Local LAN
      ↓ 실패
2. Direct P2P / WebRTC
      ↓ 실패
3. Relay
```

 또는 WebRTC의 ICE 절차를 이용하여 local candidate → STUN-discovered candidate → TURN/Relay 순으로 연결할 수 있다.

 최종적으로 사용자에게는 연결 방식이 노출되지 않아야 한다.

 예:

```
[ PC 연결 ]

QR Code Scan

       ↓

Connecting...

       ↓

Connected ✓
```

---

 # 4\. Phase 1: LAN 연결

 같은 Wi-Fi/LAN에 있는 경우에는 최대한 단순하게 연결한다.

```
Mobile
   │
   │ LAN
   ▼
Desktop
```

 Desktop은 로컬 API를 실행한다.

 예:

```
http://192.168.x.x:<port>
```

 단, 모바일이 Desktop의 IP를 직접 입력하도록 하지 않는다.

 대신 LAN discovery를 사용한다.

 후보:

 - mDNS / Bonjour
- UDP broadcast
- BLE advertisement
- QR Code

 권장:

 - Desktop → mDNS service advertisement
- Mobile → mDNS discovery
- QR Code → fallback/manual pairing

---

 # 5\. Pairing

 첫 연결은 QR Code 기반으로 한다.

 Desktop이 최초 실행되면:

```
Desktop

Device ID:
<random device id>

Pairing Code:
<short-lived code>

QR:
<QR CODE>
```

 QR에는 장기 secret을 넣지 않는다.

 가능한 payload:

```
{
  "version": 1,
  "device_id": "...",
  "pairing_token": "...",
  "endpoint_hint": "..."
}
```

 `pairing_token`은:

 - 짧은 TTL
- 1회 사용
- 사용 후 폐기
- 충분한 entropy
- brute-force 방어

 를 적용한다.

---

 # 6\. Cryptographic Identity

 Desktop과 Mobile은 각각 최초 실행 시 자체 key pair를 생성한다.

 예:

```
Desktop
  private_key_D
  public_key_D

Mobile
  private_key_M
  public_key_M
```

 Private key는 서버에 업로드하지 않는다.

 Pairing이 완료되면 각 기기에 상대방의 public key를 저장한다.

 Desktop:

```
Trusted devices:

mobile_id
mobile_public_key
```

 Mobile:

```
Trusted devices:

desktop_id
desktop_public_key
```

---

 # 7\. Application-level Authentication

 네트워크 위치나 URL만으로 신뢰하지 않는다.

 모든 요청/메시지는 application-level authentication을 사용한다.

 개념:

```
message
+
timestamp
+
nonce
+
sender identity
+
signature
```

 수신자는:

 1. sender public key 확인
2. signature 검증
3. timestamp 검증
4. nonce replay 방지
5. 권한 확인

 후 메시지를 처리한다.

---

 # 8\. Replay Attack 방지

 각 메시지에 다음을 포함한다.

```
timestamp
nonce
message_id
```

 수신자는:

 - 허용 시간 범위를 벗어난 메시지 거부
- 이미 처리한 nonce/message\_id 거부

 를 수행한다.

---

 # 9\. 데이터 암호화

 TLS/WebRTC transport encryption에 의존하는 것과 별개로,\
 민감한 application payload에 application-level encryption을 고려한다.

 목표:

```
Mobile
   │
   │ encrypted payload
   ▼
Backend / Relay
   │
   │ encrypted payload
   ▼
Desktop
```

 Relay/Backend는 실제 application payload를 해독할 수 없어야 한다.

 다만 실제 암호화 프로토콜은 직접 설계하지 말고 검증된 암호화 라이브러리/프로토콜을 사용한다.

---

 # 10\. Signaling Server

 Signaling Server의 역할:

 - device rendezvous
- session negotiation
- WebRTC SDP 전달
- ICE candidate 전달
- pairing metadata 전달
- 연결 상태 관리

 Signaling Server는 사용자 데이터를 저장하지 않는다.

 예:

```
Mobile
  │
  │ "I want to connect to Desktop X"
  ▼
Signaling Server
  │
  │ SDP / ICE
  ▼
Desktop
```

 WebRTC 연결이 성립한 이후에는 signaling server가 데이터 경로에 들어가지 않는다.

---

 # 11\. WebRTC

 WebRTC DataChannel을 P2P transport로 검토한다.

```
Mobile
   │
   │ WebRTC DataChannel
   │
   ▼
Desktop
```

 사용 목적:

 - API request/response
- realtime events
- synchronization
- small/medium data transfer
- file transfer

 대용량 파일은 별도 chunking/backpressure 전략을 검토한다.

---

 # 12\. NAT Traversal

 NAT 환경을 고려해야 한다.

 사용자는 다음 환경에 있을 수 있다.

 - 일반 home router
- CGNAT
- corporate network
- mobile carrier network
- restrictive firewall

 따라서 STUN/TURN 또는 equivalent relay mechanism을 고려한다.

 일반적인 흐름:

```
LAN candidate
    ↓
Server-reflexive candidate
    ↓
Relay candidate
```

 P2P 연결이 가능하면 P2P를 사용한다.

 불가능하면 Relay로 fallback한다.

---

 # 13\. Relay

 Relay는 최후의 fallback이다.

```
Mobile
   │
   │ encrypted
   ▼
Relay
   │
   │ encrypted
   ▼
Desktop
```

 Relay 서버는 가능한 한 다음만 수행한다.

 - connection forwarding
- rate limiting
- connection lifetime management
- bandwidth limiting
- abuse prevention

 Relay는 application payload를 해독하지 않는다.

---

 # 14\. Cloudflare 검토

 Cloudflare는 다음 역할로 검토한다.

 ### Option A: Cloudflare Tunnel

 Desktop:

```
Local API
    ↓
cloudflared
    ↓
Cloudflare
```

 Mobile:

```
Mobile
    ↓
Cloudflare
    ↓
Desktop
```

 장점:

 - inbound port 불필요
- NAT 환경에서 편리
- HTTPS endpoint 제공
- 빠른 MVP 구현 가능

 단점:

 - Cloudflare 계정/domain/credential lifecycle 관리가 제품 설계의 일부가 됨
- 각 사용자별 tunnel 관리가 복잡해질 수 있음
- 기본적으로 P2P가 아닌 Cloudflare 경유 구조
- 사용자별 endpoint 및 인증 관리 필요

 따라서 Cloudflare Tunnel을 최종 transport protocol로 강하게 결합하지 않는 것을 권장한다.

---

 # 15\. Cloudflare를 사용하는 현실적인 방법

 초기 MVP에서는:

```
Mobile
   ↓
Cloudflare
   ↓
Desktop
```

 구조로 빠르게 external connectivity를 구현할 수 있다.

 이후:

```
Mobile
   ↓
WebRTC
   ↓
Desktop
```

 을 추가하고,

 P2P 실패 시:

```
Mobile
   ↓
Relay
   ↓
Desktop
```

 으로 fallback한다.

 즉 Cloudflare는 transport abstraction 뒤에 숨긴다.

---

 # 16\. Transport Abstraction

 Application code가 특정 네트워크 기술을 직접 알지 않도록 한다.

 예:

```
Transport
├── LocalTransport
├── WebRTCTransport
├── RelayTransport
└── CloudflareTransport (optional)
```

 Application:

```
ConnectionManager
        │
        ▼
Transport interface
        │
   ┌────┼─────┐
   ▼    ▼     ▼
 LAN  WebRTC Relay
```

 이렇게 하면 향후 Tailscale, Headscale, custom relay 등을 추가할 수 있다.

---

 # 17\. Backend 최소화

 Backend에 저장하지 않는 것:

 - 사용자 문서
- 파일
- 앱 데이터
- database
- private key
- application payload

 Backend가 일시적으로 보유할 수 있는 것:

 - device ID
- ephemeral session ID
- connection state
- signaling data
- short-lived pairing information

 가능하면 TTL 기반으로 자동 삭제한다.

---

 # 18\. 계정 시스템

 기본적으로 사용자 계정을 만들지 않는다.

 Identity는:

```
Device Identity
```

 기반으로 한다.

 즉:

```
User Account
    X

Device Identity
    O
```

 사용자가 여러 기기를 연결하려면:

```
Desktop A
    │
    ├── Mobile A
    ├── Mobile B
    └── Desktop B
```

 처럼 device-to-device trust relationship을 관리한다.

---

 # 19\. Device Revocation

 사용자가 모바일을 잃어버렸을 경우를 고려해야 한다.

 Desktop UI:

```
Trusted Devices

✓ iPhone
✓ Android Tablet
✓ MacBook

[ Revoke ]
```

 Revocation하면 해당 public key를 삭제한다.

 이후 해당 device는 인증 실패.

---

 # 20\. Local API 보안

 Desktop의 localhost API도 인증 없이 공개하지 않는다.

 나쁜 구조:

```
0.0.0.0:8080
    ↓
No authentication
```

 권장:

```
Local API
    ↓
Authentication middleware
    ↓
Authorized device
```

 LAN에서 직접 접근할 경우에도 pairing/authentication을 요구한다.

---

 # 21\. Desktop Server Binding

 기본값:

```
127.0.0.1
```

 외부 연결 기능이 활성화될 때만 적절한 interface에 bind한다.

 가능하면 OS firewall rule도 최소화한다.

---

 # 22\. Connection State

 Desktop:

```
OFFLINE
LOCAL_ONLY
PAIRING
ONLINE
CONNECTED
REVOKED
```

 Mobile:

```
NOT_PAIRED
PAIRING
PAIRING_COMPLETE
CONNECTING
CONNECTED
DISCONNECTED
REVOKED
```

---

 # 23\. QR Pairing Security

 QR Code는 장기 credential이 아니다.

 권장:

```
pairing_token
TTL = 1~5 minutes
single-use
high entropy
```

 QR이 노출되어도 짧은 시간 후 무효화된다.

 가능하면 QR scan 후 사용자에게 confirmation을 요구한다.

```
Connect to:

MacBook Pro

[ Cancel ] [ Confirm ]
```

---

 # 24\. Threat Model

 고려해야 할 공격자:

 ### A. 인터넷에서 endpoint를 스캔하는 공격자

 대응:

 - 인증 필수
- rate limiting
- no unauthenticated API
- randomized identifiers
- replay protection

 ### B. QR Code를 훔쳐본 공격자

 대응:

 - short-lived pairing token
- single-use token
- explicit confirmation
- device identity verification

 ### C. Relay 서버가 악성인 경우

 대응:

 - end-to-end application encryption
- relay는 payload 해독 불가

 ### D. Mobile device 분실

 대응:

 - device revocation
- key deletion
- optional local PIN/biometric protection

 ### E. Desktop compromise

 Desktop 자체가 이미 공격받은 경우에는 암호화만으로 보호할 수 없다.

---

 # 25\. UX 목표

 사용자가 이해해야 할 것은 최소화한다.

 Desktop:

```
1. Install
2. Start app
3. QR Code displayed
```

 Mobile:

```
1. Install
2. Scan QR
3. Confirm
4. Connected
```

 사용자가 알아야 할 것:

 - IP 주소 ❌
- Port ❌
- NAT ❌
- VPN ❌
- Cloudflare ❌
- Tailscale ❌

---

 # 26\. MVP 권장 순서

 ## Phase 1

 LAN-only:

```
Mobile ←→ Desktop
```

 구현:

 - local API
- mDNS discovery
- QR pairing
- device keys
- authentication

 ## Phase 2

 Cloudflare Tunnel:

```
Mobile → Cloudflare → Desktop
```

 구현:

 - external connectivity
- HTTPS
- authentication
- pairing

 ## Phase 3

 WebRTC:

```
Mobile ←→ Desktop
```

 구현:

 - signaling
- ICE
- STUN
- DataChannel

 ## Phase 4

 Relay:

```
Mobile → Relay → Desktop
```

 구현:

 - TURN or custom relay
- bandwidth limits
- abuse protection

 ## Phase 5

 Transport optimization:

```
LAN
 ↓
P2P
 ↓
Relay
```

 자동 선택.

---

 # 27\. 핵심 설계 원칙

 1. Application은 transport를 몰라야 한다.
2. Device identity를 user account보다 우선한다.
3. Private key는 절대 서버에 저장하지 않는다.
4. Pairing token은 short-lived/single-use로 만든다.
5. URL/endpoint 자체를 credential로 취급하지 않는다.
6. 모든 remote request는 인증한다.
7. Relay는 application payload를 해독할 수 없어야 한다.
8. 중앙 서버에는 사용자 데이터를 저장하지 않는다.
9. LAN → P2P → Relay 순으로 직접 연결을 선호한다.
10. Cloudflare/Tailscale에 애플리케이션 로직을 강하게 결합하지 않는다.

---

 # 28\. 주요 기술 후보

 ## Discovery

 - mDNS
- BLE
- QR Code

 ## Transport

 - HTTP/HTTPS
- WebSocket
- WebRTC DataChannel

 ## NAT Traversal

 - STUN
- TURN
- WebRTC ICE

 ## Relay

 - TURN
- Custom encrypted relay
- Cloudflare 기반 relay 검토

 ## Identity

 - Ed25519 또는 플랫폼에 적합한 검증된 public-key primitive
- OS secure storage / Keychain / Keystore

 ## Backend

 최소한의 stateless/ephemeral signaling service.

---

 # 29\. 중요한 미결정 사항

 구현 전에 다음을 결정해야 한다.

 ### Q1. Mobile ↔ Desktop 데이터의 크기

 - 단순 command/API?
- realtime synchronization?
- 대용량 파일?
- video/audio?

 ### Q2. 연결 빈도

 - 필요할 때만 연결?
- 항상 connected?
- background connection?

 ### Q3. Mobile OS

 - iOS
- Android
- 둘 다

 ### Q4. Desktop OS

 - Windows
- macOS
- Linux

 ### Q5. Relay 운영

 - 프로젝트 운영자가 relay 제공
- 사용자가 자체 relay 제공
- Cloudflare 사용
- TURN server 사용

 ### Q6. Privacy 목표

 - TLS 수준이면 충분?
- relay도 데이터를 절대 볼 수 없어야 함?
- 완전한 E2E encryption 요구?

---

 # 30\. 검토 요청

 이 아키텍처를 실제 오픈소스 제품에 적용하기 전에 다음을 중점적으로 검토한다.

 1. WebRTC가 적절한 transport인가?
2. Cloudflare Tunnel을 MVP에 사용하는 것이 합리적인가?
3. TURN/custom relay 중 어떤 것이 적절한가?
4. 계정 없는 device identity 모델에 보안 문제가 없는가?
5. QR pairing protocol에 MITM 취약점이 없는가?
6. device revocation을 어떻게 설계해야 하는가?
7. iOS/Android background networking 제약을 어떻게 해결할 것인가?
8. Desktop이 NAT/CGNAT 뒤에 있을 때 모든 주요 네트워크 환경에서 동작하는가?
9. Relay가 악성인 경우에도 application payload를 보호할 수 있는가?
10. 중앙 서버 없이 가능한 부분과 최소한의 서버가 필요한 부분을 명확히 분리했는가?
11. 구현 복잡도가 Tailscale/Cloudflare를 직접 사용하는 것보다 과도하지 않은가?
12. 오픈소스 프로젝트의 운영/비용 측면에서 장기적으로 지속 가능한가?

 특히 \*\*"회원가입 없음 + 클라우드 데이터 저장 없음 + 외부 접속 + 보안 + 낮은 운영비"\*\*를 동시에 만족시키는 현실적인 아키텍처인지 평가한다.
