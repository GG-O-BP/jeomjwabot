# 닷패드 (Dot Pad)

## 개요

- 제조: Dot Inc
- 종류: 세계 최초 상용 **촉각 그래픽 점자 디스플레이**
- **텍스트 영역 20셀 + 그래픽 영역 300셀(10×30 격자)**, 모든 셀이 8핀
- 점자봇이 **후속 사이클**에 본격 활용할 단말기 — 그래픽 영역에 "덜 요약된 채팅 + 후원 닉네임/금액/내용 시각 강조"를 표현하기 위함

## 하드웨어 사양

| 항목 | 값 |
|---|---|
| 텍스트 영역 | 20셀 (한 줄 점자, Grade 1 표준) |
| 그래픽 영역 | 300셀 (10×30 격자), 셀당 8핀 → 가로 30 × 세로 40 도트 |
| 통신 | Bluetooth BLE + USB-C (분리 포트, 데이터·전원 분리) |
| Wi-Fi | 없음 |

## SDK 옵션

| SDK | 플랫폼 | 언어 | VoiceOver와 동시 사용 | 메모 |
|---|---|---|---|---|
| **Apple AxBrailleMap** | iOS / iPadOS 15.2+ | Swift (Cocoa Framework 내장) | ✓ 가능 | VoiceOver가 텍스트 20셀을 점유, 앱은 **그래픽 300셀만** 제어. iOS 권장 경로. |
| **DotPadFramework** | iOS 8+ | Swift | ✗ 불가 | VoiceOver 끄고 BLE 직접 점유. 텍스트+그래픽 풀 컨트롤. 시각장애 1차 사용자 워크플로우와 충돌. |
| Dot Android SDK | 1.1.0 / 2.0.0 | Kotlin | — | 공식 SDK 존재. iOS만큼 성숙하지 않음. |
| Dot Windows SDK | 1.0.0 ~ 1.4.0 | C# | — | 20셀 / dotpad 시리즈 |
| Dot Linux SDK | 1.0.0 | — | — | 한정 지원 |
| Dot Web SDK | 1.0.0 | — | — | 한정 지원 |

## VoiceOver 충돌 정책 (iOS 핵심 제약)

DotPadFramework와 VoiceOver는 **같은 BLE 채널을 점유**하므로 동시 사용 불가. 따라서 점자봇 iOS 앱은 다음 중 하나를 선택해야 한다.

1. **AxBrailleMap 경로 (권장)**
   - VoiceOver 켠 채로 그래픽 영역만 SDK로 그림
   - 텍스트 20셀은 길 A(VoiceOver)로 자동 출력 → 한소네/이모션과 동일한 ARIA 코드 재사용
   - 그래픽 영역만 별도 IPC로 송신
2. **DotPadFramework 경로**
   - VoiceOver 끄고 텍스트+그래픽 모두 직접 제어
   - 시각장애인 1차 사용자가 OS 전반에서 VoiceOver를 의존하므로 거의 채택 안 함

→ 점자봇은 **AxBrailleMap 경로**로 간다. 즉 iOS에서 닷패드는 "텍스트는 길 A, 그래픽만 SDK"로 두 채널이 공존.

## NVDA 네이티브 지원 (참고용)

NVDA PR #17007에서 Dot Pad를 시리얼 프로토콜로 직접 지원. **BrlTty의 상수·구조를 차용한 reverse-engineered 프로토콜**이며 Dot Inc 공식 SDK 사용은 아님.

- 코드: `source/brailleDisplayDrivers/dotPad/{__init__.py, defs.py}` + `source/tactile/braille.py`
- 패킷: SYNC1, SYNC2, length(big-endian 2B), destination, command hi/lo, sequence, data, XOR checksum

이 NVDA 드라이버는 **PC 환경**용. 점자봇 모바일 앱은 iOS/Android SDK 경로를 사용한다.

## 데이터 포맷

- 텍스트: SDK의 `BrailleText` API가 약자/풀자 점역 (Grade 1 표준 수준)
- 그래픽: 이미지를 8핀 셀 격자로 다운샘플링 → 점 비트맵 송신

## 점자봇 후속 사이클 작업 항목

1. **모바일 SDK 브릿지**
   - Tauri 모바일 플러그인 훅으로 Swift `AxBrailleMap` / Kotlin Dot Android SDK 호출
   - Rust 측 진입점은 `trait TactileGraphicSink { async fn render(&self, frame: GraphicFrame) -> Result<...> }` 같은 추상.
2. **VoiceOver 충돌 정책 구현**
   - iOS: AxBrailleMap만 사용 (텍스트는 화면리더, 그래픽만 SDK)
   - Android: BLE 단일 점유 정책 확인 후 결정
3. **점 비트맵 레이아웃 엔진**
   - 10×30 셀 격자에 닉네임 / 금액 / 메시지를 배치
   - 한소네/이모션의 "한 줄 잘라쓰기"와 차원이 다른 새로운 디자인 문제
   - 후원 강조: 점 패턴 박스 / 굵은 도트 / 구분선 등으로 시각 강조
4. **요약 정책 분기**
   - `shared::SummaryRequest`에 출력 매체별 길이·구조 프로필
   - 닷패드 그래픽용 "덜 요약된" 모드 — 채팅 N개를 그대로 격자에 배치하고 LLM은 핵심 강조만 표시
5. **후원 강조 디자인**
   - 닉네임 / 금액 / 내용을 시각 강조로 분리
   - 1차 사용자 검증 — 시각 사용자 추측 금지

## 셀 폭 정책 (1단계 — 길 A로 텍스트 영역만 쓸 때)

- 텍스트 20셀 = 한국어 약 10자
- 세 단말기 중 가장 빡빡 — 닷패드를 1단계에서 함께 커버하려면 요약 길이를 20셀에 맞춰야 함
- 그래픽 영역(2단계)에서는 셀 폭 제약이 사라지고 **레이아웃이 새 변수**가 됨

## 출처

- [Dot Pad Developer Center](https://developer.dotincorp.com/)
- [Dot Pad Developer FAQ](https://developer.dotincorp.com/faq/)
- [dotpad-sdk-guide (GitHub)](https://github.com/dotincorp/dotpad-sdk-guide)
- [NVDA PR #17007 — Native Dot Pad support](https://github.com/nvaccess/nvda/pull/17007)
- [Dot Pad — The first tactile graphics display](https://www.dotincorp.com/en/product/dotpadx)
- [Dot Inc Announces World's First Tactile Braille Display (PRNewswire)](https://www.prnewswire.com/news-releases/dot-inc-announces-the-worlds-first-tactile-braille-display-compatible-with-iphone-and-ipad-301500795.html)
- [Dot Pad tactile display (TechCrunch)](https://techcrunch.com/2022/03/10/dot-pad-tactile-display-makes-images-touchable-for-visually-impaired-users/)
