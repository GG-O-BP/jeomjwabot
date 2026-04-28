# 브레일 이모션 (HIMS Braille eMotion 40)

## 개요

- 제조: Selvas BLV (브랜드: HIMS International)
- 종류: 40셀 "스마트" 점자 디스플레이 (노트테이커형)
- 한소네 6와 같은 시리얼 프로토콜 가족이지만 **셀 폭이 더 넓고(40) 멀티 디바이스 페어링이 강점**
- 점좌봇 입장에서 가장 "그냥 되는" 단말기 — 표준 호환 화면리더 폭이 가장 넓음

## 하드웨어 사양

| 항목 | 값 |
|---|---|
| 점자 셀 | 40셀 |
| USB | USB-C |
| 무선 | Bluetooth (5채널 동시 페어링) |
| 동시 연결 | USB 1 + Bluetooth 5 = 최대 6 장치 |

## 입력 컨트롤

브레일 이모션도 한소네 6와 마찬가지로 **양방향 노트테이커**다. 컨트롤이 단말기 4면에 분산 배치된 점이 특징.

| 위치 | 컨트롤 |
|---|---|
| 상단 패널 | 퍼킨스 키보드, 스페이스바, Alt, Ctrl, 전원 버튼, 센터 버튼, 스크롤 버튼, Wi-Fi · 2.4GHz Wireless · 스마트 커넥트 버튼 |
| 전면 가장자리 | L/R 커서, F1~F4, Home |
| 좌측 가장자리 | 음성 옵션, 볼륨 조절 |
| 우측 가장자리 | **녹음 버튼** |

| 컨트롤 | 점좌봇 매핑 후보 |
|---|---|
| 퍼킨스 키보드 + 스페이스/Alt/Ctrl | 채팅 입력, 단축키 조합 |
| F1~F4 | 필터 토글·모드 전환 |
| L/R 커서 | 이전/다음 요약 페이지 |
| 커서 라우팅 키 (40셀 위) | **채팅 입력 오타 수정·인용 1tap 점프** |
| Home | 가장 최근 요약·최상단으로 복귀 |
| 센터 버튼 | "다시 읽기" / 자동 출력 토글 후보 |
| 스마트 커넥트 | 멀티 페어링 채널 전환 (단말 차원) |
| 음성 옵션 / 볼륨 | TTS 제어 (단말 또는 OS 차원) |
| 녹음 버튼 | 단말기 자체 녹음과 충돌 가능 — 점좌봇이 따로 매핑하기 전 사용자 의도 확인 필요 |

> 한소네 6 문서의 "단축키 매핑 함의"는 이모션에도 그대로 적용된다. F키·스크롤·미디어 등은 OS 키 이벤트로 도달하므로 `aria-keyshortcuts`로 매핑 노출.

## 통신 프로토콜

HIMS 시리얼 점자 프로토콜 패밀리 — NVDA `brailleDisplayDrivers/hims.py`가 BrailleEdge 40과 **동일 드라이버**로 처리한다.

- **USB**: Selvas BLV 다운로드 센터에서 USB 드라이버 설치 (BrailleEdge 40용 드라이버와 호환)
- **Bluetooth**: 드라이버 불필요. SPP로 자동 인식
- 시리얼 파라미터·패킷 포맷은 [한소네 6 문서](braillesense-6.md)와 동일 (115200 bps, 10바이트 고정 패킷)

## 화면리더 호환

공식 호환 명단:

| 플랫폼 | 화면리더 |
|---|---|
| Windows | JAWS, NVDA, SuperNova, Microsoft Narrator |
| macOS | VoiceOver |
| iOS / iPadOS | VoiceOver |
| Android | TalkBack |

→ 점좌봇 모바일 앱(iOS·Android)은 OS 페어링 후 별도 작업 없이 동작.

## 점좌봇과의 연동 (길 A — OS 화면리더 경유)

한소네 6와 코드 경로 동일. ARIA 기반 출력만으로 자동 동작.

```
Leptos view! → role="log" + aria-live="polite"
        ↓
iOS VoiceOver / Android TalkBack
        ↓ (USB / Bluetooth SPP)
브레일 이모션 40셀 점자 디스플레이
```

## 다중 페어링 (참고)

브레일 이모션은 BT 5채널 동시 페어링이 가능 → 사용자가 PC·모바일·태블릿을 동시에 연결하고 단말기 키 조작으로 채널 전환할 수 있다. 점좌봇은 모바일 채널 하나만 점유하면 충분하므로, 사용자가 다른 작업과 병행하기 좋다.

## 셀 폭 정책

- 40셀 = 한국어 약 20자
- `shared::SummaryRequest::max_braille_cells = 40`
- 한소네 6(32셀)보다 한 줄에 더 많은 정보 → LLM 프롬프트는 단말기별로 분기해 길이 가이드 차등 부여
- 닷패드 텍스트(20셀)와 셀 폭 차이가 2배 → "닷패드 호환" 모드를 켜면 이모션도 20셀 기준으로 줄여 일관성 확보 (사용자 옵션)

## 출처

- [Braille eMotion (Selvas BLV)](https://selvasblv.com/product/braille-emotion/)
- [브레일 이모션 (힘스인터내셔널 한국)](https://himsintl.com/kr/blindness/view.php?idx=33)
- [Braille eMotion (HIMS International)](https://himsintl.com/en/blindness/view.php?idx=34)
- [Braille eMotion 40 User Manual PDF](https://1lowvision.com/image/catalog/Flyers%20PDFs/HIMS/BrailleEMotion/Braille-eMotion-User-Manual-V1.0-HIMSInc.pdf)
- [BrailleEdge 40 (Selvas BLV)](https://selvasblv.com/support/download-center/brailleedge-40/)
- [BrailleEdge 40 USB Driver (HIMS Support)](https://hims-support.com/kb/brailleedge-40-installing-usb-driver/)
- [Introducing the Braille eMotion 40 (Sight and Sound)](https://www.sightandsound.co.uk/introducing-the-braille-emotion-40/)
- [NVDA hims.py braille driver source](https://github.com/nvaccess/nvda/blob/master/source/brailleDisplayDrivers/hims.py)
