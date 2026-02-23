# Complex Incidents

Real incidents don't come with labels. You get a page at 3 AM, a wall of alerts,
and a Slack channel full of people asking "is it fixed yet?" The difference
between a good incident responder and a panicking one is **process**.


## The Incident Lifecycle

Every incident follows the same arc, whether it takes 5 minutes or 5 hours:

1. **Alert** -- Something triggered. A monitor, a customer report, a colleague.
2. **Triage** -- How bad is it? Who's affected? What's the blast radius?
3. **Diagnose** -- What's actually broken? (Not what _seems_ broken.)
4. **Fix** -- Apply the minimum change to restore service.
5. **Verify** -- Prove it's actually fixed. Don't trust "it looks okay."
6. **Document** -- Write it down while it's fresh. Future you will thank you.


## Triage: Separate Symptoms from Causes

When multiple things are broken, your first instinct is to fix everything at once.
Resist that. Ask instead:

- **What's the _root_ cause?** Three services being down doesn't mean three problems.
  Maybe one database config is wrong and everything else is a cascade.
- **What's the _impact_?** A broken internal dashboard is not as urgent as a broken
  checkout page.
- **What can wait?** Not everything needs fixing right now. Triage ruthlessly.

The error logs are your primary tool. Read them in **chronological order** -- the
first error is usually the cause; everything after it is a symptom.


## Root Cause Analysis

The root cause is the thing that, if you fixed _only_ it, everything else would
recover. Symptoms masquerade as causes. A common trap:

```
App is returning 500s         <-- symptom
  Cache is returning errors   <-- symptom
    Database is unreachable   <-- ROOT CAUSE
```

Always ask: "Why did _this_ break?" Then ask again. Keep going until you hit
something that doesn't have a "why" above it.


## Communication During Incidents

Real incidents involve people. Even in a simulated environment, build the habit:

- **Declare the incident.** "We have an outage affecting checkout."
- **Assign a lead.** One person makes decisions. Everyone else executes.
- **Update regularly.** Every 15 minutes, even if it's "still investigating."
- **Separate investigation from communication.** Don't make the person debugging
  also be the person answering questions in Slack.


## Post-Mortems

After every significant incident, write a post-mortem. Not to assign blame --
to learn. A good post-mortem has:

- **Summary**: One paragraph. What happened, when, how long, who was affected.
- **Timeline**: Chronological list of events. When was the alert? When did
  someone start investigating? When was the fix deployed?
- **Root cause**: The actual underlying problem. Not "the server crashed" but
  "the database migration script had a bug that corrupted the sessions table."
- **Action items**: Concrete, assignable tasks to prevent recurrence. Not
  "be more careful" but "add a pre-deploy config validator to CI pipeline."

The post-mortem is not punishment. It's an investment in reliability. Teams that
skip post-mortems repeat their incidents.


## Red Herrings

Complex incidents are full of misleading signals:

- High CPU doesn't always mean a runaway process -- it might be the _symptom_ of
  a memory leak causing constant garbage collection.
- A failing health check doesn't mean the service is broken -- it might mean the
  health check endpoint itself has a bug.
- "Nothing changed" is almost never true. Something always changed. Check the
  deploy logs, the config management history, the cron jobs.

Trust the data, not the narrative. Read the logs. Check the timestamps. Follow
the evidence.
