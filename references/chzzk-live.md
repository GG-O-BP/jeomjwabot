# Live

라이브 API로 전체 라이브 목록 조회, 방송 스트림키 조회, 방송 설정 조회, 방송 설정을 변경할 수 있습니다.\
라이브 API 중 아래 API Scope를 호출하려면 사용자 계정으로 인증하여 얻은 Access Token이 필요합니다.\
API Scope는 `방송 스트림키 조회`, `방송 설정 조회`, `방송 설정 변경`입니다.

***

## 라이브 목록 조회

현재 진행 중인 라이브의 전체 목록을 조회할 수 있습니다.\
라이브 목록 조회 API를 호출하려면 애플리케이션 등록 후 Client 인증이 필요합니다. ([Client 인증 API 참조](https://chzzk.gitbook.io/chzzk/chzzk-api/tips#client-api))

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/lives</strong></td><td>라이브 목록 조회</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Param**

<table><thead><tr><th width="134">Field</th><th width="130">Type</th><th width="119">Required</th><th>Description</th></tr></thead><tbody><tr><td>size</td><td>Int</td><td>Optional</td><td>조회할 라이브 개수. 최소 1 ~ 최대 20 요청 가능<br>디폴트 값 : 20</td></tr><tr><td>next</td><td>String</td><td>Optional</td><td>다음 목록을 호출하기 위한 값입니다. api 응답 중 page.next 값을 통해 호출 가능</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="265">Field</th><th width="119">Type</th><th>Description</th></tr></thead><tbody><tr><td>data</td><td>Object[]</td><td>라이브 목록 결과. 시청자 수 높은 순 정렬</td></tr><tr><td>    liveId </td><td>Int</td><td>라이브 식별자</td></tr><tr><td>    liveTitle </td><td>String</td><td>라이브 제목</td></tr><tr><td>    liveThumbnailImageUrl </td><td>String</td><td>라이브 썸네일로 사용되는 이미지 URL</td></tr><tr><td>    concurrentUserCount </td><td>Int</td><td>라이브 현재 시청자 수</td></tr><tr><td>    openDate </td><td>String</td><td>라이브 시작 시간</td></tr><tr><td>    adult </td><td>boolean</td><td>연령 제한 설정 여부</td></tr><tr><td>    tags </td><td>String[]</td><td>라이브에 설정된 태그 목록</td></tr><tr><td>    categoryType </td><td>String</td><td><p>카테고리 종류</p><ul><li>GAME</li><li>SPORTS</li><li>ETC</li></ul></td></tr><tr><td>    liveCategory</td><td>String</td><td>라이브 카테고리 식별자</td></tr><tr><td>    liveCategoryValue</td><td>String</td><td>라이브 카테고리 이름</td></tr><tr><td>    channelId</td><td>String</td><td>채널 ID(채널 식별자)</td></tr><tr><td>    channelName</td><td>String</td><td>채널명</td></tr><tr><td>    channelImageUrl</td><td>String</td><td>채널 이미지 URL</td></tr><tr><td>page</td><td>Object</td><td></td></tr><tr><td>    next</td><td>String</td><td>다음 목록 호출을 위한 값</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 방송 스트림키 조회

스트리밍 채널의 스트림키를 조회할 수 있습니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/streams/key</strong></td><td>방송 스트림키 조회</td></tr><tr><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="264">Field</th><th width="123">Type</th><th>Description</th></tr></thead><tbody><tr><td>streamKey</td><td>String</td><td>스트림키 값</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 방송 설정 조회

스트리밍 채널의 방송 설정을 조회할 수 있습니다.

<table><thead><tr><th width="263">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>GET /open/v1/lives/setting</strong></td><td>방송 설정 조회</td></tr><tr><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="263">Field</th><th width="123">Type</th><th>Description</th></tr></thead><tbody><tr><td>defaultLiveTitle</td><td>String</td><td>라이브 제목</td></tr><tr><td>category</td><td>Object</td><td>라이브 카테고리</td></tr><tr><td>    categoryType </td><td>String</td><td><p>카테고리 종류</p><ul><li>GAME</li><li>SPORTS</li><li>ETC</li></ul></td></tr><tr><td>    categoryId</td><td>String</td><td>카테고리 식별자</td></tr><tr><td>    categoryValue</td><td>String</td><td>카테고리 이름</td></tr><tr><td>    posterImageUrl</td><td>String</td><td>카테고리 이미지 URL</td></tr><tr><td>tags </td><td>String[]</td><td>라이브 태그 목록</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 방송 설정 변경

스트리밍 채널의 방송 설정을 변경할 수 있습니다.\
설정 변경 시 API 요청을 통해 필요한 특정 값만 변경하는 것도 가능합니다.

<table><thead><tr><th width="371">HTTP Request</th><th>Description</th></tr></thead><tbody><tr><td><strong>PATCH /open/v1/lives/setting</strong></td><td>방송 설정 변경</td></tr><tr><td></td><td></td></tr></tbody></table>

**Request Body**

<table><thead><tr><th width="163">Field</th><th width="103">Type</th><th width="105">Required</th><th>Description</th></tr></thead><tbody><tr><td>defaultLiveTitle</td><td>String</td><td>optional</td><td>라이브 제목. 빈 값으로 설정 불가</td></tr><tr><td>categoryType </td><td>String</td><td>optional</td><td><p>카테고리 종류. 유효한 카테고리 종류로만 설정 가능</p><ul><li>GAME</li><li>SPORTS</li><li>ETC</li></ul></td></tr><tr><td>categoryId</td><td>String</td><td>optional</td><td>카테고리 식별자. 유효한 카테고리 종류로만 설정 가능. ""으로 전송할 경우 카테고리 설정 제거</td></tr><tr><td>tags </td><td>String[]</td><td>optional</td><td>라이브 태그 목록. empty list로 전송할 경우 태그 설정 제거. 공백 및 특수문자 비허용</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>


---

# Agent Instructions: Querying This Documentation

If you need additional information that is not directly available in this page, you can query the documentation dynamically by asking a question.

Perform an HTTP GET request on the current page URL with the `ask` query parameter:

```
GET https://chzzk.gitbook.io/chzzk/chzzk-api/live.md?ask=<question>
```

The question should be specific, self-contained, and written in natural language.
The response will contain a direct answer to the question and relevant excerpts and sources from the documentation.

Use this mechanism when the answer is not explicitly present in the current page, you need clarification or additional context, or you want to retrieve related documentation sections.
