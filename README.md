# Mufin
***This will help you connect your IoT project with Naver Clova***

## Request, Response procedure
### Requesting :
1. Clova sends IntentRequest to the local server
2. The local server requests the IntentRequest to Clovex Server on Clova's request
3. Clovex handles the request 
### Responding : 
1. Clovex sends the response to the local server
2. The local server handles the response, and sends the result back to clova
3. Clova speaks the response