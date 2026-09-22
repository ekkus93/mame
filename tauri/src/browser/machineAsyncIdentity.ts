export type MachineAsyncIdentity = {
  shortName: string | null;
  detailGeneration: number;
  activationGeneration: number;
};

export function selectMachineIdentity(
  state: MachineAsyncIdentity,
  shortName: string | null,
): MachineAsyncIdentity {
  if (state.shortName === shortName) return state;
  return {
    shortName,
    detailGeneration: state.detailGeneration + 1,
    activationGeneration: state.activationGeneration + 1,
  };
}

export function detailMayCommit(
  state: MachineAsyncIdentity,
  generation: number,
  shortName: string,
): boolean {
  return state.detailGeneration === generation && state.shortName === shortName;
}

export function activationMayCommit(
  state: MachineAsyncIdentity,
  detailGeneration: number,
  activationGeneration: number,
  shortName: string,
): boolean {
  return (
    detailMayCommit(state, detailGeneration, shortName) &&
    state.activationGeneration === activationGeneration
  );
}
