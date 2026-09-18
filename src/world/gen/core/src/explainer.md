## How this mess works
It's basically a combination of a DAG and a work-stealing queue. The general idea is to have calls to the chunk generation
system always be doing something useful without doubling up on work. This is achieved by having a global queue of jobs,
along with a local queue for each "worker". In this context a "worker" isn't a background thread or anything, each call to the
generator is a "worker" until the chunk it cares about is generated. This is done in a few steps:
1. First the list of dependency chunks is recursively expanded [here](gen_scheduler::SchedulerState::register_request).
   This is based on each stage's [`StageSpec`]. For example, if a stage says it needs the same chunk at an earlier stage
   and all neighbouring chunks at some other stage, the scheduler turns that into a bunch of [`gen_scheduler::JobKey`]s.
   Those job keys form a DAG where edges point from a dependency to the stage that becomes available after it completes.
2. Each job is shared globally. If two callers ask for overlapping chunk generation, they should end up interested in the
   same job entries instead of generating the same chunk stage twice. Each job tracks its current state, how many
   dependencies still need to complete, which jobs depend on it, and which requests are waiting for it.
3. Jobs with no remaining dependencies are published as ready work. They go into the global ready queue and also into the
   local ready queue for every request interested in them. The global queue means some caller can still make progress if
   its own target is blocked, while the local queue means a caller usually works on jobs that help its own requested chunk.
4. A call to `generate_to` loops until the requested chunk reaches the requested stage. Each loop tries to claim a ready
   job, runs that generator stage, marks the job complete, and then checks whether that completion unlocked more jobs. If
   there is no ready work, it waits for one of the jobs it cares about to wake it up.
5. Running a job means loading or creating the target chunk, collecting any required neighbour snapshots, and passing that
   into [`ChunkGenerator::advance_stage`] as a [`StageInput`]. The generator only gets mutable access to the target chunk.
   Neighbours are read-only snapshots so cross-chunk stages can look at nearby terrain without holding a bunch of chunk
   locks while they generate.

The important thing is that the scheduler does not know how terrain works. It only knows about stages and dependencies.
Generators describe their own stage graph through [`ChunkGenerator::stage_spec`], and the world wrapper uses that to make
sure a chunk is only advanced when the required earlier stages and neighbour stages exist.

There are two queues because they solve slightly different problems. The local queue is there so a request for chunk `(10, 0)`
does not spend all its time helping some unrelated chunk on the other side of the world. The global queue is there so work
that is already known to be ready does not sit around uselessly if the local request happens to be blocked. Ready jobs are
prioritized so higher-stage, closer-to-target work is preferred first.

This is still synchronous from the caller's point of view. A call to `generate_to` does not return until the requested stage
exists or generation fails. The scheduler just makes that wait more useful by letting the caller run dependency jobs while it
waits, and by sharing those jobs with other callers that ask for overlapping generation.
