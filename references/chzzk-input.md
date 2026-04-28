# Chat

채팅 API로 채팅 전송, 채팅 공지 등록, 채팅 설정 조회, 채팅 설정 변경을 할 수 있습니다.\
채팅 API를 호출하려면 사용자 계정으로 인증하여 얻은 Access Token이 필요합니다.\
API Scope는 `채팅 메시지 조회`, `채팅 메시지 쓰기`, `채팅 공지 쓰기`, `채팅 설정 조회`, `채팅 설정 변경`입니다.

***

## 채팅 메시지 전송

채팅 메시지를 전송할 수 있습니다.

관련 Scope : 채팅 메시지 쓰기

<table><thead><tr><th width="262">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/chats/send</strong></td><td>채팅 메시지 전송</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Header**

`Content-Type : application/json`

**Request Body**

<table><thead><tr><th width="134">Field</th><th width="126">Type</th><th>Description</th></tr></thead><tbody><tr><td>message</td><td>String</td><td>전송할 메시지 내용. 메시지 내용은 최대 100자로 제한</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="134">Field</th><th width="129">Type</th><th>Description</th></tr></thead><tbody><tr><td>messageId</td><td>String</td><td>전송된 메시지 ID</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 채팅 공지 등록

채팅 공지사항을 등록할 수 있습니다.\
신규 메시지 또는 전송된 기존메시지로 공지사항을 등록이 가능합니다.

관련 Scope : 채팅 공지 쓰기

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/chats/notice</strong></td><td>채팅 공지사항 등록</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Header**

`Content-Type : application/json`

**Request Body**

<table><thead><tr><th width="134">Field</th><th width="131">Type</th><th width="110">Required</th><th>Description</th></tr></thead><tbody><tr><td>message</td><td>String</td><td>Optional</td><td>신규 메시지로 공지사항 등록 시 전송할 메시지 내용. 메시지 내용은 최대 100자로 제한</td></tr><tr><td>messageId</td><td>String</td><td>Optional</td><td>기존 메시지로 공지사항 등록 시 사용하는 전송된 메시지 ID</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

**Response**

<table><thead><tr><th width="266">Code</th><th>Description</th></tr></thead><tbody><tr><td>200</td><td>공지사항 등록 성공</td></tr><tr><td></td><td></td></tr></tbody></table>

## 채팅 설정 조회

채널의 채팅 설정을 조회할 수 있습니다.

관련 Scope : 채팅 설정 조회

<table><thead><tr><th width="269">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/chats/settings</strong></td><td>채팅 설정 조회</td></tr><tr><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="290">Field</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>chatAvailableCondition</td><td>String</td><td><p>본인인증 여부 설정 조건</p><ul><li>NONE (제한 없음)</li><li>REAL_NAME (네이버 본인인증한 시청자만 채팅 허용)</li></ul></td></tr><tr><td>chatAvailableGroup</td><td>String</td><td><p>채팅 참여 범위 설정 조건</p><ul><li>ALL (모든 시청자)</li><li>FOLLOWER (팔로워 전용)</li><li>MANAGER (운영자 전용)</li><li>SUBSCRIBER (구독자 전용)</li></ul></td></tr><tr><td>minFollowerMinute</td><td>Int</td><td>FOLLOWER 모드 설정된 경우 적용된 최소 팔로잉 기간 조건</td></tr><tr><td>allowSubscriberInFollowerMode</td><td>boolean</td><td>FOLLOWER 모드 설정된 경우 구독자는 최소 팔로잉 기간 조건 대상에서 제외 허용 할지 여부</td></tr><tr><td>chatSlowModeSec</td><td>Integer</td><td>시청자의 채팅 전송 간격 (초)</td></tr><tr><td>chatEmojiMode</td><td>Boolean</td><td>이모티콘 모드 사용 여부</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 채팅 설정 변경

채널의 채팅 설정을 변경할 수 있습니다.

관련 Scope : 채팅 설정 변경

<table><thead><tr><th width="271">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>PUT /open/v1/chats/settings</strong></td><td>채팅 설정 변경</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Header**

`Content-Type : application/json`

**Request Body**

<table><thead><tr><th width="300">Field</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>chatAvailableCondition</td><td>String</td><td><p>본인인증 여부 설정 조건</p><ul><li>NONE (제한 없음)</li><li>REAL_NAME (네이버 본인인증한 시청자만 채팅 허용)</li></ul></td></tr><tr><td>chatAvailableGroup</td><td>String</td><td><p>채팅 참여 범위 설정 조건</p><ul><li>ALL (모든 시청자)</li><li>FOLLOWER (팔로워 전용)</li><li>MANAGER (운영자 전용)</li><li>SUBSCRIBER (구독자 전용)</li></ul></td></tr><tr><td>minFollowerMinute</td><td>Int</td><td>FOLLOWER 모드 설정된 경우 적용된 최소 팔로잉 기간 조건<br>0, 5, 10, 30, 60, 1440, 10080, 43200, 86400, 129600, 172800, 216000, 259200 값만 허용</td></tr><tr><td>allowSubscriberInFollowerMode</td><td>boolean</td><td>FOLLOWER 모드 설정된 경우 구독자는 최소 팔로잉 기간 조건 대상에서 제외 허용 할지 여부</td></tr><tr><td>chatSlowModeSec</td><td>int</td><td><p>시청자의 채팅 전송 간격 (초) </p><ul><li>0 (저속모드 Off), 3, 5, 10, 30, 60, 120, 300 값만 허용</li></ul></td></tr><tr><td>chatEmojiMode</td><td>boolean</td><td>이모티콘 모드 사용 여부</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 채팅 메시지 숨기기

채팅 메시지를 숨길 수 있습니다.

관련 Scope : 채팅 메시지 쓰기

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>POST /open/v1/chats/blind-message</strong></td><td>채팅 공지사항 등록</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Header**

`Content-Type : application/json`

**Request Body**

<table><thead><tr><th width="134">Field</th><th width="131">Type</th><th>Description</th></tr></thead><tbody><tr><td>chatChannelId</td><td>String</td><td>채팅 channelId</td></tr><tr><td>messageTime</td><td>long</td><td>채팅 메시지 전송 시각 (timestamp)</td></tr><tr><td>senderChannelId</td><td>String</td><td>채팅 메시지 작성자 channelId</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response**

<table><thead><tr><th width="266">Code</th><th>Description</th></tr></thead><tbody><tr><td>200</td><td>메시지 숨기기 성공</td></tr><tr><td>400</td><td>스트리머가 아닙니다.</td></tr><tr><td>403</td><td>권한 없음</td></tr><tr><td>404</td><td>해당 메시지를 찾을 수 없습니다.</td></tr><tr><td></td><td></td></tr></tbody></table>


---

# Agent Instructions: Querying This Documentation

If you need additional information that is not directly available in this page, you can query the documentation dynamically by asking a question.

Perform an HTTP GET request on the current page URL with the `ask` query parameter:

```
GET https://chzzk.gitbook.io/chzzk/chzzk-api/chat.md?ask=<question>
```

The question should be specific, self-contained, and written in natural language.
The response will contain a direct answer to the question and relevant excerpts and sources from the documentation.

Use this mechanism when the answer is not explicitly present in the current page, you need clarification or additional context, or you want to retrieve related documentation sections.
