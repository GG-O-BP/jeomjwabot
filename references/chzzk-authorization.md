# Authorization

치지직 Open API 사용과 인증에 관련된 문서입니다.\
가이드에 작성된 API 명세와 인증 플로우는 이후 개발 상황에 따라 변경 될 수 있습니다.

***

## 인증 코드 요청 및 발급

치지직 Access Token 발급을 위한 인증 코드(Authorization Code)를 요청합니다.\
요청 redirectUri 로 Access Token 발급을 위한 code 와 state 가 전달됩니다.\
인증 코드를 요청할 도메인은 아래와 같으며, OPEN API와는 다른 별도의 도메인을 사용합니다.

\*주의사항:  요청 redirectUri 는 애플리케이션 등록시 입력한 로그인 리디렉션 URL 과 일치해야합니다.<br>

**URL Path**

```
GET https://chzzk.naver.com/account-interlock
```

**Request Param**

<table><thead><tr><th width="134">Key</th><th width="122">Type</th><th width="109">Required</th><th>Example</th></tr></thead><tbody><tr><td>clientId</td><td>String</td><td>*</td><td>fefb6bbb-00c2-497c-afc2-XXXXXXXXXXXX</td></tr><tr><td>redirectUri</td><td>String</td><td>*</td><td>http://localhost:8080/api/path</td></tr><tr><td>state</td><td>String</td><td>*</td><td>zxclDasdfA25</td></tr><tr><td></td><td></td><td></td><td></td></tr></tbody></table>

**Response Parameter**

<table><thead><tr><th width="172">Key</th><th width="124">Type</th><th>Example</th></tr></thead><tbody><tr><td>code</td><td>String</td><td>ygKEQQk3p0DjUsBjJradJmXXXXXXXX</td></tr><tr><td>state</td><td>String</td><td>zxclDasdfA25</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 치지직 Access Token 발급

Open API 사용 중, 유저 인증을 위한 토큰을 발급 받습니다.\
Access Token 의 만료기간은 1일, Refresh Token 의 만료기간은 30일 입니다.<br>

**URL Path**

```
POST /auth/v1/token
```

**Request Body**

<table><thead><tr><th width="164">Key</th><th width="124">Type</th><th>Example</th></tr></thead><tbody><tr><td>grantType</td><td>String</td><td>authorization_code 고정</td></tr><tr><td>clientId</td><td>String</td><td>fefb6bbb-00c2-497c-afc2-XXXXXXXXXXXX</td></tr><tr><td>clientSecret</td><td>String</td><td>VeIMuc9XGle7PSxIVYNwPpI2OEk_9gXoW_XXXXXXXXX</td></tr><tr><td>code</td><td>String</td><td>ygKEQQk3p0DjUsBjJradJmXXXXXXXX</td></tr><tr><td>state</td><td>String</td><td>zxclDasdfA25</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="165">Key</th><th width="125">Type</th><th>Example</th></tr></thead><tbody><tr><td>accessToken</td><td>String</td><td>FFok65zQFQVcFvH2eJ7SS7SBFlTXt0EZ10L5XXXXXXXX</td></tr><tr><td>refreshToken</td><td>String</td><td>NWG05CKHAsz4k4d3PB0wQUV9ugGlp0YuibQ4XXXXXXXX</td></tr><tr><td>tokenType</td><td>String</td><td>Bearer 고정</td></tr><tr><td>expiresIn</td><td>String</td><td>86400</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 치지직 Access Token 갱신

Access Token은 만료 주기를 갖습니다. 해당 만료 주기가 지나면 해당 Access Token을 사용한 API 호출은 401(INVALID\_TOKEN) 응답을 반환합니다. \
Access Token이 만료되면, Refresh Token을 통하여 Access Token을 재발급 받아 사용해야 합니다.

Refresh Token은 Access Token 보다 긴 만료기간을 가지며, 일회용으로 사용됩니다. \
Refresh Token 또한 만료되면 Access Token 발급 과정을 통해 새로운 Access Token을 발급받아야 합니다.<br>

**URL Path**

```
POST /auth/v1/token
```

**Request Body**

<table><thead><tr><th width="160">Key</th><th width="118">Type</th><th>Example</th></tr></thead><tbody><tr><td>grantType</td><td>String</td><td>refresh_token 고정</td></tr><tr><td>refreshToken</td><td>String</td><td>NWG05CKHAsz4k4d3PB0wQUV9ugGlp0YuibQ4XXXXXXXX</td></tr><tr><td>clientId</td><td>String</td><td>fefb6bbb-00c2-497c-afc2-XXXXXXXXXXXX</td></tr><tr><td>clientSecret</td><td>String</td><td>VeIMuc9XGle7PSxIVYNwPpI2OEk_9gXoW_XXXXXXXXX</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

**Response Body**

<table><thead><tr><th width="158">Key</th><th width="118">Type</th><th>Example</th></tr></thead><tbody><tr><td>accessToken</td><td>String</td><td>motTJ-NZ-fev3cmaTMydzYk_zyw524C9ZYdNXXXXXXXX</td></tr><tr><td>refreshToken</td><td>String</td><td>EDpM_1RxiOwhbNBpNUbiuEZOrb7Dbd6Y7rivXXXXXXXX</td></tr><tr><td>tokenType</td><td>String</td><td>Bearer 고정</td></tr><tr><td>expiresIn</td><td>String</td><td>86400</td></tr><tr><td>scope</td><td>String</td><td>채널 조회</td></tr><tr><td></td><td></td><td></td></tr></tbody></table>

## 치지직 Access Token 삭제

유저가 로그아웃하는 등, 해당 Access Token, Refresh Token 의 revoke 가 필요할 경우 호출합니다. \
요청한 Token 과 동일한 인증 과정을 거친 모든 Token 이 제거됩니다. (clientId 와 user 가 동일한 Token)<br>

**URL Path**

```
POST /auth/v1/token/revoke
```

**Request Body**

<table><thead><tr><th width="166">Key</th><th width="112">Type</th><th>Example</th></tr></thead><tbody><tr><td>clientId</td><td>String</td><td>fefb6bbb-00c2-497c-afc2-XXXXXXXXXXXX</td></tr><tr><td>clientSecret</td><td>String</td><td>VeIMuc9XGle7PSxIVYNwPpI2OEk_9gXoW_XXXXXXXXX</td></tr><tr><td>token</td><td>String</td><td>motTJ-NZ-fev3cmaTMydzYk_zyw524C9ZYdNXXXXXXXX</td></tr><tr><td>tokenTypeHint</td><td>String</td><td><ul><li>access_token (default)</li><li>refresh_token</li></ul></td></tr><tr><td></td><td></td><td></td></tr></tbody></table>


---

# Agent Instructions: Querying This Documentation

If you need additional information that is not directly available in this page, you can query the documentation dynamically by asking a question.

Perform an HTTP GET request on the current page URL with the `ask` query parameter:

```
GET https://chzzk.gitbook.io/chzzk/chzzk-api/authorization.md?ask=<question>
```

The question should be specific, self-contained, and written in natural language.
The response will contain a direct answer to the question and relevant excerpts and sources from the documentation.

Use this mechanism when the answer is not explicitly present in the current page, you need clarification or additional context, or you want to retrieve related documentation sections.
