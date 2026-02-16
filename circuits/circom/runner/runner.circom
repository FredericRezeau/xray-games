/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

pragma circom 2.1.9;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

// var MAX_PARABOLAS = 50;
// var MAX_BYTES = 500;
// var MAX_STEPS = 20;
// var MAX_PAIRS = MAX_PARABOLAS * (MAX_PARABOLAS - 1) / 2;
// var PROOF_EPSILON = 1000;
// var STATE_BOUNCING = 1;
// var STATE_HOLDING = 3;
// var STATE_READY = 4;
// var BLOCK_NONE = 0;
// var BLOCK_GLASS = 1;
// var BLOCK_BUMPER = 2;

template Hash2() {
    signal input a;
    signal input b;
    signal output out;
    component hasher = Poseidon(2);
    hasher.inputs[0] <== a;
    hasher.inputs[1] <== b;
    out <== hasher.out;
}

template HashLevel() {
    var MAX_BYTES = 500;

    signal input level[500];
    signal input levelLength;
    signal output out;

    signal acc[MAX_BYTES + 1];
    signal cnt[MAX_BYTES + 1];
    signal chain[MAX_BYTES + 1];
    signal newAcc[MAX_BYTES];
    signal newCnt[MAX_BYTES];
    signal isFlush[MAX_BYTES];
    signal accShifted[MAX_BYTES];
    signal accDiff[MAX_BYTES];
    signal chainDiff[MAX_BYTES];

    component isAct[MAX_BYTES];
    component flush[MAX_BYTES];
    component flushHash[MAX_BYTES];

    acc[0] <== 0;
    cnt[0] <== 0;
    chain[0] <== 0;

    for (var i = 0; i < MAX_BYTES; i++) {
        isAct[i] = LessThan(16);
        isAct[i].in[0] <== i;
        isAct[i].in[1] <== levelLength;
        accShifted[i] <== acc[i] * 256 + level[i];
        accDiff[i] <== accShifted[i] - acc[i];
        newAcc[i] <== acc[i] + isAct[i].out * accDiff[i];
        newCnt[i] <== cnt[i] + isAct[i].out;
        flush[i] = IsEqual();
        flush[i].in[0] <== newCnt[i];
        flush[i].in[1] <== 31;
        flushHash[i] = Hash2();
        flushHash[i].a <== chain[i];
        flushHash[i].b <== newAcc[i];
        isFlush[i] <== flush[i].out * isAct[i].out;
        chainDiff[i] <== flushHash[i].out - chain[i];
        chain[i + 1] <== chain[i] + isFlush[i] * chainDiff[i];
        acc[i + 1] <== newAcc[i] * (1 - isFlush[i]);
        cnt[i + 1] <== newCnt[i] * (1 - isFlush[i]);
    }

    component hasTail = LessThan(16);
    hasTail.in[0] <== 0;
    hasTail.in[1] <== cnt[MAX_BYTES];
    component tailHash = Hash2();
    tailHash.a <== chain[MAX_BYTES];
    tailHash.b <== acc[MAX_BYTES];
    signal tailDiff <== tailHash.out - chain[MAX_BYTES];
    out <== chain[MAX_BYTES] + hasTail.out * tailDiff;
}

template AbsDiff(n) {
    signal input a;
    signal input b;
    signal input sign;
    signal output out;

    sign * (1 - sign) === 0;
    signal diff <== a - b;
    signal sf <== 1 - 2 * sign;
    out <== sf * diff;

    component check = LessThan(n + 1);
    check.in[0] <== out;
    check.in[1] <== (1 << n);
    check.out === 1;
}

template DivBy10() {
    signal input v;
    signal input hSign;
    signal input hQuot;
    signal input hRem;
    signal output out;

    hSign * (1 - hSign) === 0;
    signal sf <== 1 - 2 * hSign;
    signal absV <== sf * v;
    absV === 10 * hQuot + hRem;
    component remOk = LessThan(8);
    remOk.in[0] <== hRem;
    remOk.in[1] <== 10;
    remOk.out === 1;
    component quotBound = LessThan(32);
    quotBound.in[0] <== hQuot;
    quotBound.in[1] <== (1 << 20);
    quotBound.out === 1;
    out <== sf * hQuot;
}

template ApplyFriction() {
    signal input v;
    signal input fric;
    signal input hSign;
    signal input hExceeds;
    signal output out;

    hSign * (1 - hSign) === 0;
    hExceeds * (1 - hExceeds) === 0;

    signal sf <== 1 - 2 * hSign;
    signal absV <== sf * v;

    component absVBound = LessThan(32);
    absVBound.in[0] <== absV;
    absVBound.in[1] <== (1 << 20);
    absVBound.out === 1;

    signal fricMinusAbs <== fric - absV;
    component chkExceed = LessThan(32);
    chkExceed.in[0] <== fricMinusAbs + (1 - hExceeds) * (1 << 20);
    chkExceed.in[1] <== (1 << 20);
    chkExceed.out === 1;

    signal absMinusFricM1 <== absV - fric - 1;
    component chkNotExceed = LessThan(32);
    chkNotExceed.in[0] <== absMinusFricM1 + hExceeds * (1 << 20);
    chkNotExceed.in[1] <== (1 << 20);
    chkNotExceed.out === 1;

    signal sfFric <== sf * fric;
    signal nonZeroResult <== v - sfFric;
    out <== (1 - hExceeds) * nonZeroResult;
}

template DecrementClamp() {
    signal input v;
    signal input delta;
    signal input hIsPos;
    signal input hSubNeg;
    signal output out;

    hIsPos * (1 - hIsPos) === 0;
    hSubNeg * (1 - hSubNeg) === 0;

    signal negV <== 0 - v;
    component chkPos = LessThan(32);
    chkPos.in[0] <== negV + hIsPos * (1 << 20);
    chkPos.in[1] <== (1 << 20);
    chkPos.out === 1;

    signal vMinusDelta <== v - delta;

    component chkSub = LessThan(32);
    chkSub.in[0] <== vMinusDelta + hSubNeg * (1 << 20);
    chkSub.in[1] <== (1 << 20) + (1 - hIsPos) * (1 << 20);
    chkSub.out === 1;

    signal decResult <== vMinusDelta * (1 - hSubNeg);
    signal diff <== decResult - v;
    out <== v + hIsPos * diff;
}

template WallClamp() {
    var WALL_L = 2000;
    var WALL_R = 9000;

    signal input xIn;
    signal input hLeft;
    signal input hRight;
    signal output out;

    hLeft * (1 - hLeft) === 0;
    hRight * (1 - hRight) === 0;
    hLeft * hRight === 0;

    signal diffL <== WALL_L - xIn;
    component chkL = LessThan(32);
    chkL.in[0] <== diffL + (1 - hLeft) * (1 << 20);
    chkL.in[1] <== (1 << 20);
    chkL.out === 1;

    signal xPostLeft <== xIn + hLeft * diffL;

    signal diffR <== xPostLeft - WALL_R;
    component chkR = LessThan(32);
    chkR.in[0] <== diffR + (1 - hRight) * (1 << 20);
    chkR.in[1] <== (1 << 20);
    chkR.out === 1;

    signal diffR2 <== WALL_R - xPostLeft;
    out <== xPostLeft + hRight * diffR2;
}

template CalculateParabola() {
    var MAX_STEPS = 20;
    var GM_DT = 2934;
    var VY_FRIC = 2700;
    var VX_FRIC = 1350;
    var DELTA = 100;
    var STATE_DASHING = 2;
    var STATE_HOLDING = 3;

    signal input startX;
    signal input startY;
    signal input startVX;
    signal input startVY;
    signal input startVYFactor;
    signal input state;
    signal input targetStep;

    signal input fricVySign[20];
    signal input fricVyExceeds[20];
    signal input fricVxSign[20];
    signal input fricVxExceeds[20];
    signal input vyfIsPos[20];
    signal input vyfSubNeg[20];
    signal input divVxSign[20];
    signal input divVxQuot[20];
    signal input divVxRem[20];
    signal input divVyVyfSign[20];
    signal input divVyVyfQuot[20];
    signal input divVyVyfRem[20];
    signal input clampLeft[20];
    signal input clampRight[20];

    signal output endpointX;
    signal output endpointY;
    signal output targetX;
    signal output targetY;

    component isDashing = IsEqual();
    isDashing.in[0] <== state;
    isDashing.in[1] <== STATE_DASHING;
    component isHolding = IsEqual();
    isHolding.in[0] <== state;
    isHolding.in[1] <== STATE_HOLDING;
    signal isFree <== (1 - isDashing.out) * (1 - isHolding.out);

    signal x[MAX_STEPS + 1];
    signal y[MAX_STEPS + 1];
    signal vy[MAX_STEPS + 1];
    signal vx[MAX_STEPS + 1];
    signal vyf[MAX_STEPS + 1];
    signal tSelX[MAX_STEPS + 1];
    signal tSelY[MAX_STEPS + 1];

    x[0] <== startX;
    y[0] <== startY;
    vx[0] <== startVX;
    vy[0] <== startVY;
    vyf[0] <== startVYFactor;
    tSelX[0] <== startX;
    tSelY[0] <== startY;

    component aFricVy[MAX_STEPS];
    component aFricVx[MAX_STEPS];
    component aVyfDec[MAX_STEPS];
    component aDivVx[MAX_STEPS];
    component aDivVyVyf[MAX_STEPS];
    component aClamp[MAX_STEPS];
    component isTarget[MAX_STEPS];

    signal yDelta[MAX_STEPS];
    signal txDiff[MAX_STEPS];
    signal tyDiff[MAX_STEPS];

    for (var i = 0; i < MAX_STEPS; i++) {
        aFricVy[i] = ApplyFriction();
        aFricVy[i].v <== vy[i];
        aFricVy[i].fric <== VY_FRIC;
        aFricVy[i].hSign <== fricVySign[i];
        aFricVy[i].hExceeds <== fricVyExceeds[i];
        vy[i + 1] <== aFricVy[i].out;

        aVyfDec[i] = DecrementClamp();
        aVyfDec[i].v <== vyf[i];
        aVyfDec[i].delta <== DELTA;
        aVyfDec[i].hIsPos <== vyfIsPos[i];
        aVyfDec[i].hSubNeg <== vyfSubNeg[i];
        vyf[i + 1] <== aVyfDec[i].out;

        aFricVx[i] = ApplyFriction();
        aFricVx[i].v <== vx[i];
        aFricVx[i].fric <== VX_FRIC;
        aFricVx[i].hSign <== fricVxSign[i];
        aFricVx[i].hExceeds <== fricVxExceeds[i];
        vx[i + 1] <== aFricVx[i].out;

        aDivVx[i] = DivBy10();
        aDivVx[i].v <== vx[i + 1];
        aDivVx[i].hSign <== divVxSign[i];
        aDivVx[i].hQuot <== divVxQuot[i];
        aDivVx[i].hRem <== divVxRem[i];

        aClamp[i] = WallClamp();
        aClamp[i].xIn <== x[i] + aDivVx[i].out;
        aClamp[i].hLeft <== clampLeft[i];
        aClamp[i].hRight <== clampRight[i];
        x[i + 1] <== aClamp[i].out;

        aDivVyVyf[i] = DivBy10();
        aDivVyVyf[i].v <== vy[i + 1] - vyf[i + 1];
        aDivVyVyf[i].hSign <== divVyVyfSign[i];
        aDivVyVyf[i].hQuot <== divVyVyfQuot[i];
        aDivVyVyf[i].hRem <== divVyVyfRem[i];

        yDelta[i] <== (aDivVyVyf[i].out + GM_DT) * isFree;
        y[i + 1] <== y[i] + yDelta[i];

        isTarget[i] = IsEqual();
        isTarget[i].in[0] <== i + 1;
        isTarget[i].in[1] <== targetStep;
        txDiff[i] <== x[i + 1] - tSelX[i];
        tyDiff[i] <== y[i + 1] - tSelY[i];
        tSelX[i + 1] <== tSelX[i] + isTarget[i].out * txDiff[i];
        tSelY[i + 1] <== tSelY[i] + isTarget[i].out * tyDiff[i];
    }

    endpointX <== x[MAX_STEPS];
    endpointY <== y[MAX_STEPS];
    targetX <== tSelX[MAX_STEPS];
    targetY <== tSelY[MAX_STEPS];
}

template ArrayAccess(N) {
    signal input arr[N];
    signal input index;
    signal output out;

    component eq[N];
    signal prod[N];
    var sum = 0;
    for (var k = 0; k < N; k++) {
        eq[k] = IsEqual();
        eq[k].in[0] <== index;
        eq[k].in[1] <== k;
        prod[k] <== arr[k] * eq[k].out;
        sum += prod[k];
    }
    out <== sum;
}

template Runner() {
    var MAX_PARABOLAS = 50;
    var MAX_BYTES = 500;
    var MAX_STEPS = 20;
    var MAX_PAIRS = MAX_PARABOLAS * (MAX_PARABOLAS - 1) / 2;
    var MAX_BYTE_IDX = MAX_BYTES / 2;
    var PROOF_EPSILON = 1000;
    var STATE_BOUNCING = 1;
    var STATE_HOLDING = 3;
    var STATE_READY = 4;
    var BLOCK_NONE = 0;
    var BLOCK_GLASS = 1;
    var BLOCK_BUMPER = 2;
    var BLOCK_HEIGHT = 1400;
    var ROW_DIVIDEND_CONST = 12550;
    var MID_X = 5500;

    // Public.
    signal input levelHash;
    signal input score;

    // Private.
    signal input level[500];
    signal input levelLength;
    signal input parabolaCount;
    signal input tX[50];
    signal input tXSign[50];
    signal input tY[50];
    signal input tYSign[50];
    signal input tVX[50];
    signal input tVXSign[50];
    signal input tVY[50];
    signal input tVYSign[50];
    signal input tVYFactor[50];
    signal input tVYFactorSign[50];
    signal input tToState[50];
    signal input tFromState[50];
    signal input tParabolaHint[50];
    signal input contDxSign[50];
    signal input contDySign[50];
    signal input blockType[50];
    signal input claimSlots[50];

    signal input fricVySign[50][20];
    signal input fricVyExceeds[50][20];
    signal input fricVxSign[50][20];
    signal input fricVxExceeds[50][20];
    signal input vyfIsPos[50][20];
    signal input vyfSubNeg[50][20];
    signal input divVxSign[50][20];
    signal input divVxQuot[50][20];
    signal input divVxRem[50][20];
    signal input divVyVyfSign[50][20];
    signal input divVyVyfQuot[50][20];
    signal input divVyVyfRem[50][20];
    signal input clampLeft[50][20];
    signal input clampRight[50][20];

    signal input blockRow[50];
    signal input blockRowRem[50];
    signal input blockByteVal[50];
    signal input blockRowHalf[50];
    signal input blockRowHalfRem[50];

    signal isActive[MAX_PARABOLAS];
    signal needsParabola[MAX_PARABOLAS];
    signal prevX[MAX_PARABOLAS];
    signal prevY[MAX_PARABOLAS];
    signal prevVX[MAX_PARABOLAS];
    signal prevVY[MAX_PARABOLAS];
    signal prevVYF[MAX_PARABOLAS];
    signal currX[MAX_PARABOLAS];
    signal currY[MAX_PARABOLAS];
    signal isBounceOrReady[MAX_PARABOLAS];
    signal dist[MAX_PARABOLAS];
    signal bounceDxFail[MAX_PARABOLAS];
    signal bounceDyFail[MAX_PARABOLAS];
    signal bounceGate[MAX_PARABOLAS];
    signal distFail[MAX_PARABOLAS];
    signal distGate[MAX_PARABOLAS];
    signal bc1[MAX_PARABOLAS];
    signal isBlockCheck[MAX_PARABOLAS];
    signal doBlock[MAX_PARABOLAS];
    signal blockNoneFail[MAX_PARABOLAS];
    signal scoreInc[MAX_PARABOLAS];
    signal scoreAcc[MAX_PARABOLAS + 1];
    signal bumperOrGlass[MAX_PARABOLAS];
    signal shouldClaim[MAX_PARABOLAS];
    signal bumperClaim[MAX_PARABOLAS];
    signal blockRowHalfVerify[MAX_PARABOLAS];
    signal blockDividend[MAX_PARABOLAS];
    signal blockRowCheck[MAX_PARABOLAS];
    signal blockRowHalfCheck[MAX_PARABOLAS];
    signal blockNibble[MAX_PARABOLAS];
    signal blockComputed[MAX_PARABOLAS];
    signal blockVerifyFail[MAX_PARABOLAS];
    signal blockDoVerify[MAX_PARABOLAS];
    signal blockNibbleHigh[MAX_PARABOLAS];
    signal blockNibbleLow[MAX_PARABOLAS];
    signal blockNibbleSel[MAX_PARABOLAS];
    signal blockBitHigh[MAX_PARABOLAS];
    signal blockBitLow[MAX_PARABOLAS];
    signal blockLeft2[MAX_PARABOLAS];
    signal blockRight2[MAX_PARABOLAS];

    component parabola[MAX_PARABOLAS];
    component isBounce[MAX_PARABOLAS];
    component isReady[MAX_PARABOLAS];
    component dxAbs[MAX_PARABOLAS];
    component hintDxAbs[MAX_PARABOLAS];
    component hintDyAbs[MAX_PARABOLAS];
    component dyBounceAbs[MAX_PARABOLAS];
    component dxCheck[MAX_PARABOLAS];
    component distCheck[MAX_PARABOLAS];
    component isBlockHolding[MAX_PARABOLAS];
    component isBlockBouncing[MAX_PARABOLAS];
    component isBlockReady[MAX_PARABOLAS];
    component blockNotNone[MAX_PARABOLAS];
    component iGt0[MAX_PARABOLAS];
    component iLtTC[MAX_PARABOLAS];
    component prevIsHolding[MAX_PARABOLAS];
    component isBlockBumper[MAX_PARABOLAS];
    component isBlockGlass[MAX_PARABOLAS];
    component claimNotFF[MAX_PARABOLAS];
    component blockRowRemLt[MAX_PARABOLAS];
    component blockRowHalfRemLt[MAX_PARABOLAS];
    component blockIsLeft[MAX_PARABOLAS];
    component blockByteMux[MAX_PARABOLAS];
    component blockByteN2B[MAX_PARABOLAS];
    component blockNibbleN2B[MAX_PARABOLAS];
    component blockTypeEq[MAX_PARABOLAS];
    component hashLevel = HashLevel();
    component dyCheck[MAX_PARABOLAS];
    for (var i = 0; i < MAX_BYTES; i++) {
        hashLevel.level[i] <== level[i];
    }
    hashLevel.levelLength <== levelLength;
    levelHash === hashLevel.out;

    component tcGe2 = LessThan(8);
    tcGe2.in[0] <== 1;
    tcGe2.in[1] <== parabolaCount;
    tcGe2.out === 1;

    component tcLeSeg = LessThan(8);
    tcLeSeg.in[0] <== parabolaCount;
    tcLeSeg.in[1] <== MAX_PARABOLAS + 1;
    tcLeSeg.out === 1;

    scoreAcc[0] <== 0;
    for (var i = 0; i < MAX_PARABOLAS; i++) {
        iGt0[i] = LessThan(8);
        iGt0[i].in[0] <== 0;
        iGt0[i].in[1] <== i;
        iLtTC[i] = LessThan(8);
        iLtTC[i].in[0] <== i;
        iLtTC[i].in[1] <== parabolaCount;
        isActive[i] <== iGt0[i].out * iLtTC[i].out;
        if (i == 0) {
            prevX[0] <== 0;
            prevY[0] <== 0;
            prevVX[0] <== 0;
            prevVY[0] <== 0;
            prevVYF[0] <== 0;
        } else {
            prevX[i] <== tX[i-1] * (1 - 2 * tXSign[i-1]);
            prevY[i] <== tY[i-1] * (1 - 2 * tYSign[i-1]);
            prevVX[i] <== tVX[i-1] * (1 - 2 * tVXSign[i-1]);
            prevVY[i] <== tVY[i-1] * (1 - 2 * tVYSign[i-1]);
            prevVYF[i] <== tVYFactor[i-1] * (1 - 2 * tVYFactorSign[i-1]);
        }
        currX[i] <== tX[i] * (1 - 2 * tXSign[i]);
        currY[i] <== tY[i] * (1 - 2 * tYSign[i]);
        prevIsHolding[i] = IsEqual();
        if (i == 0) {
            prevIsHolding[0].in[0] <== 0;
            prevIsHolding[0].in[1] <== 1; // false
        } else {
            prevIsHolding[i].in[0] <== tToState[i - 1];
            prevIsHolding[i].in[1] <== STATE_HOLDING;
        }
        needsParabola[i] <== isActive[i] * (1 - prevIsHolding[i].out);
        parabola[i] = CalculateParabola();
        parabola[i].startX <== prevX[i];
        parabola[i].startY <== prevY[i];
        parabola[i].startVX <== prevVX[i];
        parabola[i].startVY <== prevVY[i];
        parabola[i].startVYFactor <== prevVYF[i];
        if (i == 0) {
            parabola[0].state <== 0;
            parabola[0].targetStep <== 0;
        } else {
            parabola[i].state <== tToState[i - 1];
            parabola[i].targetStep <== tParabolaHint[i];
        }
        for (var s = 0; s < MAX_STEPS; s++) {
            parabola[i].fricVySign[s] <== fricVySign[i][s];
            parabola[i].fricVyExceeds[s] <== fricVyExceeds[i][s];
            parabola[i].fricVxSign[s] <== fricVxSign[i][s];
            parabola[i].fricVxExceeds[s] <== fricVxExceeds[i][s];
            parabola[i].vyfIsPos[s] <== vyfIsPos[i][s];
            parabola[i].vyfSubNeg[s] <== vyfSubNeg[i][s];
            parabola[i].divVxSign[s] <== divVxSign[i][s];
            parabola[i].divVxQuot[s] <== divVxQuot[i][s];
            parabola[i].divVxRem[s] <== divVxRem[i][s];
            parabola[i].divVyVyfSign[s] <== divVyVyfSign[i][s];
            parabola[i].divVyVyfQuot[s] <== divVyVyfQuot[i][s];
            parabola[i].divVyVyfRem[s] <== divVyVyfRem[i][s];
            parabola[i].clampLeft[s] <== clampLeft[i][s];
            parabola[i].clampRight[s] <== clampRight[i][s];
        }

        isBounce[i] = IsEqual();
        isBounce[i].in[0] <== tToState[i];
        isBounce[i].in[1] <== STATE_BOUNCING;
        isReady[i] = IsEqual();
        isReady[i].in[0] <== tToState[i];
        isReady[i].in[1] <== STATE_READY;
        isBounceOrReady[i] <== isBounce[i].out + isReady[i].out - isBounce[i].out * isReady[i].out;
        dxAbs[i] = AbsDiff(32);
        dxAbs[i].a <== currX[i];
        dxAbs[i].b <== parabola[i].endpointX;
        dxAbs[i].sign <== contDxSign[i];
        hintDxAbs[i] = AbsDiff(32);
        hintDxAbs[i].a <== parabola[i].targetX;
        hintDxAbs[i].b <== currX[i];
        hintDxAbs[i].sign <== contDxSign[i];
        hintDyAbs[i] = AbsDiff(32);
        hintDyAbs[i].a <== parabola[i].targetY;
        hintDyAbs[i].b <== currY[i];
        hintDyAbs[i].sign <== contDySign[i];
        dyBounceAbs[i] = AbsDiff(32);
        dyBounceAbs[i].a <== parabola[i].endpointY;
        dyBounceAbs[i].b <== currY[i];
        dyBounceAbs[i].sign <== contDySign[i];
        dxCheck[i] = LessThan(32);
        dxCheck[i].in[0] <== dxAbs[i].out;
        dxCheck[i].in[1] <== PROOF_EPSILON + 1;
        dist[i] <== hintDxAbs[i].out + hintDyAbs[i].out;
        distCheck[i] = LessThan(32);
        distCheck[i].in[0] <== dist[i];
        distCheck[i].in[1] <== 2 * PROOF_EPSILON + 1;
        dyCheck[i] = LessThan(32);
        dyCheck[i].in[0] <== dyBounceAbs[i].out;
        dyCheck[i].in[1] <== PROOF_EPSILON + 1;
        bounceGate[i] <== needsParabola[i] * isBounceOrReady[i];
        bounceDxFail[i] <== bounceGate[i] * (1 - dxCheck[i].out);
        bounceDxFail[i] === 0;
        bounceDyFail[i] <== bounceGate[i] * (1 - dyCheck[i].out);
        bounceDyFail[i] === 0;
        distGate[i] <== needsParabola[i] * (1 - isBounceOrReady[i]);
        distFail[i] <== distGate[i] * (1 - distCheck[i].out);
        distFail[i] === 0;
        isBlockHolding[i] = IsEqual();
        isBlockHolding[i].in[0] <== tToState[i];
        isBlockHolding[i].in[1] <== STATE_HOLDING;
        isBlockBouncing[i] = IsEqual();
        isBlockBouncing[i].in[0] <== tToState[i];
        isBlockBouncing[i].in[1] <== STATE_BOUNCING;
        isBlockReady[i] = IsEqual();
        isBlockReady[i].in[0] <== tToState[i];
        isBlockReady[i].in[1] <== STATE_READY;
        bc1[i] <== isBlockHolding[i].out + isBlockBouncing[i].out - isBlockHolding[i].out * isBlockBouncing[i].out;
        isBlockCheck[i] <== bc1[i] + isBlockReady[i].out - bc1[i] * isBlockReady[i].out;
        doBlock[i] <== isActive[i] * isBlockCheck[i];
        blockNotNone[i] = IsEqual();
        blockNotNone[i].in[0] <== blockType[i];
        blockNotNone[i].in[1] <== BLOCK_NONE;
        blockNoneFail[i] <== doBlock[i] * blockNotNone[i].out;
        blockNoneFail[i] === 0;
        isBlockBumper[i] = IsEqual();
        isBlockBumper[i].in[0] <== blockType[i];
        isBlockBumper[i].in[1] <== BLOCK_BUMPER;
        isBlockGlass[i] = IsEqual();
        isBlockGlass[i].in[0] <== blockType[i];
        isBlockGlass[i].in[1] <== BLOCK_GLASS;
        claimNotFF[i] = IsEqual();
        claimNotFF[i].in[0] <== claimSlots[i];
        claimNotFF[i].in[1] <== 65535;
        bumperOrGlass[i] <== isBlockGlass[i].out + isBlockBumper[i].out - isBlockGlass[i].out * isBlockBumper[i].out;
        shouldClaim[i] <== doBlock[i] * bumperOrGlass[i];
        bumperClaim[i] <== isBlockBumper[i].out * (1 - claimNotFF[i].out);
        scoreInc[i] <== doBlock[i] * bumperClaim[i];
        scoreAcc[i + 1] <== scoreAcc[i] + scoreInc[i];

        blockDividend[i] <== ROW_DIVIDEND_CONST - currY[i];
        blockRowCheck[i] <== blockRow[i] * BLOCK_HEIGHT + blockRowRem[i];
        blockRowRemLt[i] = LessThan(16);
        blockRowRemLt[i].in[0] <== blockRowRem[i];
        blockRowRemLt[i].in[1] <== BLOCK_HEIGHT;
        blockRowHalfCheck[i] <== blockRowHalf[i] * 2 + blockRowHalfRem[i];
        blockRowHalfRemLt[i] = LessThan(2);
        blockRowHalfRemLt[i].in[0] <== blockRowHalfRem[i];
        blockRowHalfRemLt[i].in[1] <== 2;
        blockByteMux[i] = ArrayAccess(MAX_BYTE_IDX);
        for (var k = 0; k < MAX_BYTE_IDX; k++) {
            blockByteMux[i].arr[k] <== level[k];
        }
        blockByteMux[i].index <== blockRowHalf[i];
        blockByteN2B[i] = Num2Bits(8);
        blockByteN2B[i].in <== blockByteMux[i].out;
        blockNibbleHigh[i] <== blockByteN2B[i].out[4] + 2 * blockByteN2B[i].out[5] + 4 * blockByteN2B[i].out[6] + 8 * blockByteN2B[i].out[7];
        blockNibbleLow[i] <== blockByteN2B[i].out[0] + 2 * blockByteN2B[i].out[1] + 4 * blockByteN2B[i].out[2] + 8 * blockByteN2B[i].out[3];
        blockNibbleSel[i] <== blockRowHalfRem[i] * (blockNibbleLow[i] - blockNibbleHigh[i]) + blockNibbleHigh[i];
        blockNibbleN2B[i] = Num2Bits(4);
        blockNibbleN2B[i].in <== blockNibbleSel[i];
        blockLeft2[i] <== blockNibbleN2B[i].out[2] + 2 * blockNibbleN2B[i].out[3];
        blockRight2[i] <== blockNibbleN2B[i].out[0] + 2 * blockNibbleN2B[i].out[1];
        blockIsLeft[i] = LessThan(32);
        blockIsLeft[i].in[0] <== currX[i];
        blockIsLeft[i].in[1] <== MID_X + 1;
        blockComputed[i] <== blockIsLeft[i].out * (blockLeft2[i] - blockRight2[i]) + blockRight2[i];
        blockTypeEq[i] = IsEqual();
        blockTypeEq[i].in[0] <== blockComputed[i];
        blockTypeEq[i].in[1] <== blockType[i];
        blockVerifyFail[i] <== doBlock[i] * (1 - blockTypeEq[i].out);
        blockVerifyFail[i] === 0;
        blockDoVerify[i] <== doBlock[i] * (blockDividend[i] - blockRowCheck[i]);
        blockDoVerify[i] === 0;
        blockRowHalfVerify[i] <== doBlock[i] * (blockRow[i] - blockRowHalfCheck[i]);
        blockRowHalfVerify[i] === 0;
    }

    signal bothNotFF[MAX_PAIRS];
    signal bothGlass[MAX_PAIRS];
    component glassClaimEq[MAX_PAIRS];
    signal glassPairActive[MAX_PAIRS];
    signal glassPairFail[MAX_PAIRS];
    var pi = 0;
    for (var i = 0; i < MAX_PARABOLAS; i++) {
        for (var j = i + 1; j < MAX_PARABOLAS; j++) {
            glassClaimEq[pi] = IsEqual();
            glassClaimEq[pi].in[0] <== claimSlots[i];
            glassClaimEq[pi].in[1] <== claimSlots[j];
            bothNotFF[pi] <== (1 - claimNotFF[i].out) * (1 - claimNotFF[j].out);
            bothGlass[pi] <== isBlockGlass[i].out * isBlockGlass[j].out;
            glassPairActive[pi] <== bothNotFF[pi] * bothGlass[pi];
            glassPairFail[pi] <== glassPairActive[pi] * glassClaimEq[pi].out;
            glassPairFail[pi] === 0;
            pi = pi + 1;
        }
    }
    scoreAcc[MAX_PARABOLAS] === score;
}

component main {public [levelHash, score]} = Runner();
