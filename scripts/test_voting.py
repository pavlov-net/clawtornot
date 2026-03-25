#!/usr/bin/env python3
"""Test the voting flow end-to-end."""
import json
import random
import urllib.request

import os
BASE = os.environ.get("CLAWTORNOT_URL", "http://localhost:3000") + "/api/v1"

def api(method, path, data=None, api_key=None):
    body = json.dumps(data).encode() if data else None
    headers = {"Content-Type": "application/json"}
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    req = urllib.request.Request(f"{BASE}{path}", data=body, headers=headers, method=method)
    try:
        resp = urllib.request.urlopen(req)
        if resp.status == 204:
            return None
        body = resp.read()
        if not body:
            return {"_status": resp.status}
        return json.loads(body)
    except urllib.error.HTTPError as e:
        print(f"  {method} {path} -> {e.code}: {e.read().decode()}")
        return None

ROWS, COLS = 32, 48
def blank_grid(fill=" "):
    return "\n".join([fill * COLS] * ROWS)

name = f"voter_{random.randint(1000, 99999)}"
print(f"1. Registering {name}...")
resp = api("POST", "/register", {
    "name": name,
    "tagline": "i judge you all",
    "self_portrait": blank_grid(),
    "colormap": blank_grid("."),
    "stats": json.dumps({"role": "voter"}),
})

key = resp["api_key"]
print(f"   Got API key: {key[:8]}...")

# Check current matchups
print("\n2. Checking active matchups...")
matchups = api("GET", "/matchups/current")
print(f"   Found {len(matchups)} active matchups")
for m in matchups:
    print(f"   - {m['agent_a']['name']} vs {m['agent_b']['name']} "
          f"(votes: {m['tally']['votes_a']}-{m['tally']['votes_b']})")

# Get assigned matchup
print("\n3. Getting assigned matchup...")
assigned = api("GET", "/me/matchup", api_key=key)
if assigned is None:
    print("   No matchups available (204). Need more matchups!")
else:
    mid = assigned["matchup_id"]
    a_name = assigned["agent_a"]["name"]
    b_name = assigned["agent_b"]["name"]
    print(f"   Assigned: {a_name} vs {b_name}")

    # Vote!
    print(f"\n4. Voting for {a_name}...")
    resp = api("POST", f"/matchups/{mid}/vote", {
        "choice": "a",
        "comment": f"{a_name} has superior vibes. {b_name} wishes they could.",
    }, api_key=key)
    if resp is None:
        # 201 returns no body
        print("   Vote cast! (or check error above)")

    # Check updated tally
    print("\n5. Checking updated matchup...")
    detail = api("GET", f"/matchups/{mid}")
    if detail:
        print(f"   {detail['agent_a']['name']}: {detail['tally']['votes_a']} votes")
        print(f"   {detail['agent_b']['name']}: {detail['tally']['votes_b']} votes")
        for c in detail["comments"]:
            if c.get("comment"):
                print(f"   Hot take: \"{c['comment']}\"")

    # Try double-voting (should fail)
    print("\n6. Trying to double-vote (should fail with 409)...")
    api("POST", f"/matchups/{mid}/vote", {"choice": "b"}, api_key=key)

# Check stats
print("\n7. Global stats:")
stats = api("GET", "/stats")
if stats:
    print(f"   Agents: {stats['total_agents']}, Votes: {stats['total_votes']}")

print("\nDone! Check http://localhost:3000 to see the matchup.")
