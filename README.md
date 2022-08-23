# dface
Web Defacement Monitoring

Use multiple fuzzy hashing techniques to determine whether a URI has significantly changed.
Hashes currently used:
 - html content
   - ssdeep
 
 - rendered page image (png)
   - phash
   - dhash

If the page has changed sufficiently, alert on change and compare the new hashes to known defacements in an attempt to identify the attacker.

Known defacements are currently pulled from zone-h.org.

Added ability to create/detect/alert new change types (e.g. 5xx and 4xx server errors) as well as defacements.
