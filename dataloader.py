import redis
import json

import random

r = redis.Redis(host='133.133.135.22', port=6379, decode_responses=True)
# r = redis.Redis(host='192.168.137.144', port=6379, decode_responses=True)

# r.set("aaa", 1)
# s = r.get("aaa")
# print(type(s))
# print(s)

with open('social.json', 'r') as f:
    data = json.load(f)
    for nodeid in data.keys():
        r.set(nodeid, json.dumps(data[nodeid]))

with open('pagerank.json', 'r') as f:
    data = json.load(f)
    r.set('pagerank', json.dumps(data))

s = "abcdefghijklmnopqrstuvwxyz1234567890"
md5 = ''
for i in range(1024 * 64):
    md5 += s[random.randint(0, len(s) - 1)]

print(md5)

r.set('md5', md5)


# r.set('name', 'runoob')
# print(r['name'])
# print(r.get('name'))
