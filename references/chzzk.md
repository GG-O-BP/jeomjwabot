# Session

세션 생성, 세션 목록 조회, 이벤트 구독 및 취소를 할 수 있습니다.\
세션 API 중 아래 API Scope를 호출하려면 사용자 계정으로 인증하여 얻은 Access Token이 필요합니다.\
API Scope는 `채팅 메시지 조회`, `후원 조회`, `구독 조회`입니다.

***

## 세션 생성(클라이언트)

Client 인증을 통해 소켓 연결을 위한 URL을 요청합니다. 생성된 URL은 일정 시간 동안만 유효합니다. \
최대 10개의 연결을 유지할 수 있습니다.\
세션 생성(클라이언트)를 호출하려면 애플리케이션 등록 후 Client 인증이 필요합니다. ([Client 인증 API 참조](https://chzzk.gitbook.io/chzzk/chzzk-api/tips#client-api))

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/sessions/auth/client</strong></td><td>세션 생성(클라이언트)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="266">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>url</td><td>String</td><td>소켓 연결을 위한 URL</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 세션 생성(유저)

Access Token 인증을 통해 소켓 연결을 위한 URL을 요청합니다. 생성된 URL은 일정 시간 동안만 유효합니다. \
연결된 세션은 세션 생성에 사용된 Access Token과 동일한 유저 이벤트만 구독할 수 있습니다. \
유저별 최대 3개의 연결을 유지할 수 있습니다

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/sessions/auth</strong></td><td>세션 생성(유저)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="266">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>url</td><td>String</td><td>소켓 연결을 위한 URL</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 세션 연결 가이드

[Socket.IO-client](https://github.com/socketio/socket.io-client) 1.0.0+ 2.0.3 버전까지 지원합니다.

**소켓 연결**

```markup
// api를 통해 얻은 연결 url
const sessionURL = 'https://ssio08.nchat.naver.com:443?auth=TOKEN';
 
// 옵션 설정
const socketOption = {
      reconnection: false,
      'force new connection': true,
      'connect timeout': 3000,
      transports: ['websocket'],
};
 
 
// ...
 
// 세션 연결
socket = io.connect(sessionURL, socketOption)
socket.on('connect', function() { 
              // on connected
       });
```

연결이 완료될 경우 세션으로 [연결 완료 메시지](#undefined-11)가 전달 됩니다. 해당 메시지의 sessionKey 값을 통해 연결된 세션에 이벤트를 구독할 수 있습니다.

**메시지 수신**

```
// eventType 메시지
socket.on("SYSTEM", function(data) {
    /* on system event */
});
```

## 세션 목록 조회(클라이언트)

Client 인증 기반의 생성된 세션을 조회합니다. 연결이 끊어진 세션은 90일 동안만 조회가 가능합니다.\
세션 목록 조회(클라이언트)를 호출하려면 애플리케이션 등록 후 Client 인증이 필요합니다. ([Client 인증 API 참조](https://chzzk.gitbook.io/chzzk/chzzk-api/tips#client-api))

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/sessions/client</strong></td><td>세션 목록 조회(클라이언트)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="266">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>size</td><td>Int</td><td><p>조회할 세션 개수. 최소 1 ~ 최대 50 요청 가능</p><p>default : 20</p></td></tr><tr><td>page</td><td>String</td><td><p>조회할 페이지. 0부터 조회 가능</p><p>default : 0</p></td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="265">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>data</td><td>Object[]</td><td>세션 목록 결과</td></tr><tr><td>    sessionKey</td><td>String</td><td>세션 식별자</td></tr><tr><td>    connectedDate</td><td>String</td><td>연결 시간</td></tr><tr><td>    disconnectedDate</td><td>String</td><td>연결 해제 시간</td></tr><tr><td>    subscribedEvents</td><td>Object[]</td><td>구독 이벤트 목록</td></tr><tr><td>        eventType</td><td>String</td><td><p>이벤트 종류</p><ul><li>CHAT</li><li>DONATION</li><li>SUBSCRIPTION</li></ul></td></tr><tr><td>        channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 세션 목록 조회(유저)

Access Token 인증 기반의 생성된 세션을 조회합니다. 연결이 끊어진 세션은 90일동안만 조회 가능합니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/sessions</strong></td><td>세션 목록 조회(유저)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="266">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>size</td><td>Int</td><td><p>조회할 세션 개수. 최소 1 ~ 최대 50 요청 가능</p><p>default : 20</p></td></tr><tr><td>page</td><td>String</td><td><p>조회할 페이지. 0부터 조회 가능</p><p>default : 0</p></td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="265">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>data</td><td>Object[]</td><td>세션 목록 결과</td></tr><tr><td>    sessionKey</td><td>String</td><td>세션 식별자</td></tr><tr><td>    connectedDate</td><td>String</td><td>연결 시간</td></tr><tr><td>    disconnectedDate</td><td>String</td><td>연결 해제 시간</td></tr><tr><td>    subscribedEvents</td><td>Object[]</td><td>구독 이벤트 목록</td></tr><tr><td>        eventType</td><td>String</td><td><p>이벤트 종류</p><ul><li>CHAT</li><li>DONATION</li><li>SUBSCRIPTION</li></ul></td></tr><tr><td>        channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

***

## 이벤트 구독(채팅)

요청한 세션에 사용자의 채팅 이벤트를 구독합니다. \
구독이 완료될 경우 요청한 세션으로 [이벤트 구독 메시지](#message-event-subscribe)가 전달 됩니다. \
채팅 이벤트 구독 시, 구독한 채널에 채팅이 발생할 때 [채팅 이벤트 메시지](#message-event-subscribe-chat)가 전달됩니다.

관련 Scope : 채팅 메시지 조회

\*세션당 최대 30개의 이벤트(채팅, 후원, 구독)를 구독할 수 있습니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/subscribe/chat</strong></td><td>이벤트 구독(채팅)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

## 이벤트 구독 취소(채팅)

요청한 세션에 사용자의 채팅 이벤트를 구독 취소합니다. \
구독이 취소될 경우 요청한 세션으로 [이벤트 구독 취소 메시지](#message-event-unsubscribe)가 전달 됩니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/unsubscribe/chat</strong></td><td>이벤트 구독 취소(채팅)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

## 이벤트 구독(후원)

요청한 세션에 사용자의 후원 이벤트를 구독합니다. \
구독이 완료될 경우 요청한 세션으로 [이벤트 구독 메시지](#message-event-subscribe)가 전달 됩니다. \
이벤트 구독 시, 구독한 채널에 후원이 발생할 때 [후원 이벤트 메시지](#message-event-subscribe-donation)가 전달됩니다.

관련 Scope : 후원 조회

\*세션당 최대 30개의 이벤트(채팅, 후원, 구독)를 구독할 수 있습니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/subscribe/donation</strong></td><td>이벤트 구독(후원)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

## 이벤트 구독 취소(후원)

요청한 세션에 사용자의 후원 이벤트를 구독 취소합니다.\
구독이 취소될 경우 요청한 세션으로 [이벤트 구독 취소 메시지](#message-event-unsubscribe)가 전달 됩니다.

<table><thead><tr><th width="266">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/unsubscribe/donation</strong></td><td>이벤트 구독 취소(후원)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

## 이벤트 구독(구독)

요청한 세션에 사용자의 구독 이벤트를 구독합니다. \
구독이 완료될 경우 요청한 세션으로 [이벤트 구독 메시지](#message-event-subscribe)가 전달 됩니다. \
이벤트 구독 시, 구독한 채널에 구독이 발생할 때 [구독 이벤트 메시지](#message-event-subscribe-subscription)가 전달됩니다.

관련 Scope : 구독 조회

\*세션당 최대 30개의 이벤트(채팅, 후원, 구독)를 구독할 수 있습니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/subscribe/subscription</strong></td><td>이벤트 구독(구독)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

## 이벤트 구독 취소(구독)

요청한 세션에 사용자의 구독 이벤트를 구독 취소합니다.\
구독이 취소될 경우 요청한 세션으로 [이벤트 구독 취소 메시지](#message-event-unsubscribe)가 전달 됩니다.

<table><thead><tr><th width="266">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/sessions/events/unsubscribe/subscription</strong></td><td>이벤트 구독 취소(구독)</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th width="138">Required</th><th>Description</th></tr></thead><tbody><tr><td>sessionKey</td><td>String</td><td>*</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

***

## 세션 메시지

세션으로 전달되는 메시지에는 시스템 메시지, 구독 이벤트 메시지가 존재합니다.

## 시스템 메시지(공통)

`Event Type : SYSTEM`

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th>Description</th></tr></thead><tbody><tr><td>type</td><td>String</td><td><p>시스템 메시지 종류</p><ul><li>connected</li><li>subscribed</li><li>unsubscribed</li><li>revoked</li></ul></td></tr><tr><td>data</td><td>Object</td><td>시스템 메시지 정보</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 시스템 메시지(연결 완료 메시지)

소켓 연결이 정상적으로 완료 되었을 때 전달됩니다. 전달된 세션 식별자를 통해 이벤트를 구독/취소 할 수 있습니다.

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th>Description</th></tr></thead><tbody><tr><td>type</td><td>String</td><td>connected</td></tr><tr><td>data</td><td>Object</td><td>시스템 메시지 정보</td></tr><tr><td>    sessionKey</td><td>String</td><td>세션 식별자</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 시스템 메시지(이벤트 구독 메시지) <a href="#message-event-subscribe" id="message-event-subscribe"></a>

이벤트 구독이 완료 되었을 때 전달됩니다.

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th>Description</th></tr></thead><tbody><tr><td>type</td><td>String</td><td>subscribed</td></tr><tr><td>data</td><td>Object</td><td>시스템 메시지 정보</td></tr><tr><td>    eventType</td><td>String</td><td><p>이벤트 종류</p><ul><li>CHAT</li><li>DONATION</li><li>SUBSCRIPTION</li></ul></td></tr><tr><td>    channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 시스템 메시지(이벤트 구독 취소 메시지) <a href="#message-event-unsubscribe" id="message-event-unsubscribe"></a>

이벤트 구독이 취소 되었을 때 전달됩니다.

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th>Description</th></tr></thead><tbody><tr><td>type</td><td>String</td><td>unsubscribed</td></tr><tr><td>data</td><td>Object</td><td>시스템 메시지 정보</td></tr><tr><td>    eventType</td><td>String</td><td><p>이벤트 종류</p><ul><li>CHAT</li><li>DONATION</li><li>SUBSCRIPTION</li></ul></td></tr><tr><td>    channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 시스템 메시지(이벤트 권한 취소 메시지)

사용자의 동의 철회, 스코프 변경 등 권한 회수로 이벤트 구독이 취소 되었을 때 전달됩니다.

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="92">Type</th><th>Description</th></tr></thead><tbody><tr><td>type</td><td>String</td><td>revoked</td></tr><tr><td>data</td><td>Object</td><td>시스템 메시지 정보</td></tr><tr><td>    eventType</td><td>String</td><td><p>이벤트 종류</p><ul><li>CHAT</li><li>DONATION</li><li>SUBSCRIPTION</li></ul></td></tr><tr><td>    channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 구독 이벤트 메시지(채팅 이벤트 메시지) <a href="#message-event-subscribe-chat" id="message-event-subscribe-chat"></a>

구독한 채널의 채팅 메시지가 전달됩니다.

`Event Type : CHAT`

**Message Body**

<table><thead><tr><th width="173">Field</th><th width="98">Type</th><th>Description</th></tr></thead><tbody><tr><td>channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td>senderChannelId</td><td>String</td><td>채팅 메시지 작성자 채널 ID</td></tr><tr><td>chatChannelId</td><td>String</td><td>채팅 메시지가 속한 채팅 채널 ID (임시제한, 메시지 삭제에 사용)</td></tr><tr><td>profile</td><td>Object</td><td>채팅 메시지 작성자 프로필 정보</td></tr><tr><td>    nickname</td><td>String</td><td>닉네임</td></tr><tr><td>    badges</td><td>Object[]</td><td>배지</td></tr><tr><td>    verifiedMark</td><td>boolean</td><td>인증여부</td></tr><tr><td>userRoleCode</td><td>String</td><td><p>유저 채널 권한</p><ul><li>streamer : 스트리머</li><li>common_user : 일반 유저</li><li>streaming_channel_manager : 채널 관리자</li><li>streaming_chat_manager : 채팅 운영자</li></ul></td></tr><tr><td>content</td><td>String</td><td>채팅 메시지 내용</td></tr><tr><td>emojis</td><td>Map</td><td>사용된 치지직 이모티콘 정보</td></tr><tr><td>    key</td><td>String</td><td>치지직 이모티콘 식별자</td></tr><tr><td>    value</td><td>String</td><td>치지직 이모티콘 URL</td></tr><tr><td>messageTime</td><td>Int64</td><td>메시지 시간 (ms)</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 구독 이벤트 메시지(후원 이벤트 메시지) <a href="#message-event-subscribe-donation" id="message-event-subscribe-donation"></a>

구독한 채널의 후원 메시지가 전달됩니다.

`Event Type : DONATION`

**Message Body**

<table><thead><tr><th width="184">Field</th><th width="98">Type</th><th>Description</th></tr></thead><tbody><tr><td>donationType</td><td>String</td><td><p>후원 종류</p><ul><li>CHAT</li><li>VIDEO</li></ul></td></tr><tr><td>channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td>donatorChannelId</td><td>String</td><td>후원자 채널 ID</td></tr><tr><td>donatorNickname</td><td>String</td><td>후원자 닉네임</td></tr><tr><td>payAmount</td><td>String</td><td>후원 금액 (원)</td></tr><tr><td>donationText</td><td>String</td><td>후원 메시지 내용</td></tr><tr><td>emojis</td><td>Map</td><td>사용된 치지직 이모티콘 정보</td></tr><tr><td>    key</td><td>String</td><td>치지직 이모티콘 식별자</td></tr><tr><td>    value</td><td>String</td><td>치지직 이모티콘 URL</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 구독 이벤트 메시지(구독 이벤트 메시지) <a href="#message-event-subscribe-subscription" id="message-event-subscribe-subscription"></a>

구독한 채널의 구독 메시지가 전달됩니다.

`Event Type : SUBSCRIPTION`

**Message Body**

<table><thead><tr><th width="184">Field</th><th width="98">Type</th><th>Description</th></tr></thead><tbody><tr><td>channelId</td><td>String</td><td>이벤트 채널 ID(채널 식별자)</td></tr><tr><td>subscriberChannelId</td><td>String</td><td>구독자 채널 ID</td></tr><tr><td>subscriberNickname</td><td>String</td><td>구독자 닉네임</td></tr><tr><td>tierNo</td><td>Int</td><td><p>구독 상품</p><ul><li>1 (티어1 구독)</li><li>2 (티어2 구독)</li></ul></td></tr><tr><td>tierName</td><td>String</td><td>구독 브랜드명</td></tr><tr><td>month</td><td>Int</td><td>사용된구독 기간치지직 이모티콘 정보</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>


---

# Agent Instructions: Querying This Documentation

If you need additional information that is not directly available in this page, you can query the documentation dynamically by asking a question.

Perform an HTTP GET request on the current page URL with the `ask` query parameter:

```
GET https://chzzk.gitbook.io/chzzk/chzzk-api/session.md?ask=<question>
```

The question should be specific, self-contained, and written in natural language.
The response will contain a direct answer to the question and relevant excerpts and sources from the documentation.

Use this mechanism when the answer is not explicitly present in the current page, you need clarification or additional context, or you want to retrieve related documentation sections.
