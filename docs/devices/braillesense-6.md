# 한소네 6 (HIMS BrailleSense 6)

## 개요

- 제조: Selvas BLV (브랜드: HIMS International)
- 종류: 32셀 점자 디스플레이 + 퍼킨스식 점자 키보드 + Android 12 기반 자체 OS의 노트테이커
- 즉 단순 점자 디스플레이가 아니라 "점자 스마트폰". 한소네 위에서 직접 앱을 돌릴 수도 있고, PC·모바일의 외부 점자 디스플레이로도 쓸 수 있다.

## 하드웨어 사양

| 항목 | 값 |
|---|---|
| 점자 셀 | 32셀 |
| 키보드 | 퍼킨스식 6-key + 기능키 |
| USB | USB-C 3.1 Gen1 Device Mode 1, USB-C 3.1 Gen1 Host 1, USB-A 2.0 Host 2 |
| 영상 | HDMI 출력 |
| 무선 | Wi-Fi, Bluetooth |
| 스토리지 / RAM | 64GB / 6GB |
| OS | Android 12 |

## 외부 점자 디스플레이로 쓰기 — Terminal for Screen Reader

한소네 6는 자체 OS를 가진 단말이라, 한소네 자체 화면리더로 점자를 출력할 수도 있고 **"Terminal for Screen Reader"** 모드를 켜서 PC·모바일의 외부 점자 디스플레이로도 동작한다. 점좌봇이 모바일 앱으로 사용자 옆에서 동작할 때 길 A의 진입점이 이 모드다.

연결 절차 (사용자 측):

1. 한소네 6에서 Terminal for Screen Reader 모드 진입
2. 모바일/PC와 USB-C 직결 또는 Bluetooth 페어링
3. 화면리더가 자동 인식 (드라이버 OS 내장)

## 통신 프로토콜 (외부 디스플레이 모드)

NVDA `brailleDisplayDrivers/hims.py` 기준으로 정리:

- **USB Bulk (Custom)**: VID `0x045E`, PID `0x930A` / `0x930B` (BrailleSense 계열)
- **USB HID**: 일부 신모델 (예: Edge 3S `0x940A`)
- **Bluetooth SPP**: 장치명 `BrailleSense*` 접두사로 자동 인식
- **시리얼 파라미터**: 115200 bps, 패리티 없음, 0.2s 타임아웃
- **패킷 포맷 (10바이트 고정)**: `[패킷타입×2][모드][0xF0][길이(2)][데이터][0xF1][체크섬]`

> ⚠ NVDA hims.py 코드의 명시 매트릭스에는 BrailleSense **6**, Polaris, Mini, QBraille이 빠져 있다. 외부 디스플레이 모드의 펌웨어가 동일 시리얼 패킷을 쓰는 것으로 인식되지만, AppleVis 포럼에서 Win11 + JAWS BT 페어링 첫 인식 후 재인식 실패 사례가 보고됨. **USB-C 직결이 가장 안정적**.

## 화면리더별 설정

| 화면리더 | 경로 |
|---|---|
| NVDA | Preferences → Settings → Braille → Add → "HIMS BrailleSense" |
| JAWS | Options → Braille → Add Braille Display → "HIMS Braille" |
| iOS VoiceOver | 시스템 Bluetooth 페어링 후 자동 (HIMS 드라이버 OS 내장) |
| Android TalkBack | 시스템 Bluetooth 페어링 또는 USB-OTG 후 자동 |

## 점좌봇과의 연동 (길 A — OS 화면리더 경유)

```
Leptos view! → role="log" + aria-live="polite"
        ↓
iOS VoiceOver / Android TalkBack
        ↓ (USB Bulk / Bluetooth SPP)
한소네 6 32셀 점자 디스플레이
```

점좌봇은 한소네 6를 **직접 제어하지 않는다**. ARIA + `aria-live`만 정확히 쓰면 화면리더가 알아서 흘려보낸다.

## 셀 폭 정책

- 32셀 = 한국어 약 16자
- `shared::SummaryRequest::max_braille_cells = 32`로 LLM 프롬프트에 반영
- 한 줄에 들어가지 않으면 의미 단위로 분할

## 출처

- [HIMS BrailleSense 6 (한국)](https://himsintl.com/kr/blindness/view.php?idx=15)
- [BrailleSense 6 (HIMS International)](https://www.himsintl.com/en/blindness/view.php?idx=8)
- [BrailleSense 6 User Manual EN PDF](https://selvasblv.com/download/bs6/BrailleSense_6_User_Manual_English_V1.0_210528.pdf)
- [BrailleSense 6 Getting Started Guide PDF](https://selvasblv.com/wp-content/uploads/2023/08/BrailleSense-6-Getting-Started-Guide-digital.pdf)
- [Connecting BrailleSense 6 to NVDA/JAWS/Narrator (AppleVis)](https://www.applevis.com/forum/windows/connecting-braillesense-6-nvdajawsnarrator)
- [Using Terminal for screen reader on HIMS BrailleSense (SAS Ltd)](https://sastltd.zohodesk.eu/portal/en/kb/articles/using-terminal-for-screen-reader-on-hims-braillesense-notetakers-18-8-2023)
- [NVDA hims.py braille driver source](https://github.com/nvaccess/nvda/blob/master/source/brailleDisplayDrivers/hims.py)
- [BRLTTY on Android](https://brltty.app/doc/Android.html)
