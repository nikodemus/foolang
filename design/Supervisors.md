# Supervisors

Every worker should be supervised.

So `Process run: Agent` is actually

```
Supervisor temporary: Agent
```

...and "agent" is any class that responds to the message
"run: <mailbox>"

```
Supervisor permanent: [Agent1, Agent2] stragegy: $oneForRest
```

Etc. How do you get access to mailboxes of Agent1 and Agent2 in the example
above?

```
supervisor := Supervise permanent: { agent1: Agent1, agent2: Agent2 }
                        # Using a constant provides error checking unlike
                        # literal!
                        strategy: OneForOne
                        intensity: 1
                        period: 5
supervisor agent1 doStuff: 42
```

Plausible.

How does the supervisor actually monitor the children and propagate
failures?

1. It's a process of it's own (heck, maybe there _is_ a lower
   level process class underneath.) It polls the kids, and
   if it needs to panic it triggers that in the parent.
   (Either through operating system / VM facilities,
   or through sending the parent a message.)

2. It's part of the parent process which has it's own message
   queue, part of which maintenance is going through supervisors.

Handwavy, but good enough.
