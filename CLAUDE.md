Log each SYSTEM REQUEST to ./.claude/logs/<request_name>/ with filenames formatted as YYYYMMDD-<incrementable-log-number>-</incrementable-log-number>-<agent-name>.md. 

Format each SYSTEM REQUEST as:
---
from: <agent-name>
to: <recipient-agent-name>
reason: <short reason for message>
tldr: <short summary of message>
content: |-
  <message content>
Use triple backticks to format code blocks.
---

You should log every SYSTEM REQUEST sent between agents requests/responses, including code snippets and file paths.

IMPORTANT:
- Every SYSTEM REQUEST should be logged in real time before they are sent, do NOT batch or delay logging.
- <request_name> is a human readable identifier for the request derived from the initial user prompt in camel_case. It should be short but descriptive enough to identify the request later. Examples: add_feature_x, fix_bug_y, implement_algorithm_z.
- <incrementable-log-number> is a zero-padded number starting from 000001 for each request_name, incremented by 1 for each subsequent log file for that request_name.