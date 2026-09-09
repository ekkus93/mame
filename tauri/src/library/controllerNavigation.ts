export type ControllerNavigationAction = "previousMachine" | "nextMachine" | "activateFocused";

export type ControllerNavigationContext = {
  documentFocused: boolean;
  gameplayActive: boolean;
};

export type ControllerNavigationState = {
  armed: boolean;
  verticalDirection: -1 | 0 | 1;
  primaryPressed: boolean;
};

export type ControllerSample = {
  connected: boolean;
  standardMapping: boolean;
  dpadUp: boolean;
  dpadDown: boolean;
  primaryPressed: boolean;
  leftStickY: number;
};

export type ControllerNavigationResult = {
  state: ControllerNavigationState;
  actions: ControllerNavigationAction[];
};

export const INITIAL_CONTROLLER_NAVIGATION_STATE: ControllerNavigationState = {
  armed: false,
  verticalDirection: 0,
  primaryPressed: false,
};

export const CONTROLLER_AXIS_ENTER_THRESHOLD = 0.7;
export const CONTROLLER_AXIS_RELEASE_THRESHOLD = 0.35;

export function canUseControllerNavigation(context: ControllerNavigationContext): boolean {
  return context.documentFocused && !context.gameplayActive;
}

export function updateControllerNavigation(
  previous: ControllerNavigationState,
  sample: ControllerSample,
  context: ControllerNavigationContext,
): ControllerNavigationResult {
  if (!canUseControllerNavigation(context) || !sample.connected || !sample.standardMapping) {
    return {
      state: INITIAL_CONTROLLER_NAVIGATION_STATE,
      actions: [],
    };
  }

  const verticalDirection = resolveVerticalDirection(sample, previous.verticalDirection);
  const neutral = verticalDirection === 0 && !sample.primaryPressed;

  if (!previous.armed) {
    return {
      state: {
        armed: neutral,
        verticalDirection,
        primaryPressed: sample.primaryPressed,
      },
      actions: [],
    };
  }

  const actions: ControllerNavigationAction[] = [];
  if (verticalDirection !== 0 && verticalDirection !== previous.verticalDirection) {
    actions.push(verticalDirection === -1 ? "previousMachine" : "nextMachine");
  }

  if (!previous.primaryPressed && sample.primaryPressed) {
    actions.push("activateFocused");
  }

  return {
    state: {
      armed: true,
      verticalDirection,
      primaryPressed: sample.primaryPressed,
    },
    actions,
  };
}

function resolveVerticalDirection(
  sample: ControllerSample,
  previousDirection: ControllerNavigationState["verticalDirection"],
): ControllerNavigationState["verticalDirection"] {
  if (sample.dpadUp && sample.dpadDown) {
    return 0;
  }
  if (sample.dpadUp) {
    return -1;
  }
  if (sample.dpadDown) {
    return 1;
  }

  if (previousDirection === -1 && sample.leftStickY <= -CONTROLLER_AXIS_RELEASE_THRESHOLD) {
    return -1;
  }
  if (previousDirection === 1 && sample.leftStickY >= CONTROLLER_AXIS_RELEASE_THRESHOLD) {
    return 1;
  }
  if (sample.leftStickY <= -CONTROLLER_AXIS_ENTER_THRESHOLD) {
    return -1;
  }
  if (sample.leftStickY >= CONTROLLER_AXIS_ENTER_THRESHOLD) {
    return 1;
  }
  return 0;
}
