/*
    This file is part of xray-games project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/
 
 pragma circom 2.1.9;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

// var MAX_POLYGONS = 4;
// var MAX_VERTICES_PER_POLYGON = 14;
// var MAX_PARTITIONS = 20;
// var MAX_VERTICES_PER_PARTITION = 14;
// var MAX_OBJECTS = 15;
// var MAX_SEGMENTS = 3;
// var TOLERANCE = 10000;

// Array mapping:
//   polygons: [4][14][2] = [MAX_POLYGONS][MAX_VERTICES_PER_POLYGON][2]
//   partitions: [20][14][2] = [MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION][2]
//   objects: [15][2] = [MAX_OBJECTS][2]
//   segments: [3][4] = [MAX_SEGMENTS][4]

template Hash2() {
    signal input a;
    signal input b;
    signal output out;

    component hasher = Poseidon(2);
    hasher.inputs[0] <== a;
    hasher.inputs[1] <== b;
    out <== hasher.out;
}

template Cross() {
    signal input p1x, p1y, p2x, p2y, p3x, p3y;
    signal output out;
    signal dx1 <== p2x - p1x;
    signal dy1 <== p2y - p1y;
    signal dx2 <== p3x - p1x;
    signal dy2 <== p3y - p1y;
    signal t1 <== dx1 * dy2;
    signal t2 <== dy1 * dx2;
    out <== t1 - t2;
}

template AbsLessThan(n) {
    signal input val;
    signal input sign;
    signal input hint;
    signal input threshold;
    signal output out;

    sign * (1 - sign) === 0;
    signal sf <== 1 - 2 * sign;
    val === sf * hint;
    component lt = LessThan(n);
    lt.in[0] <== hint;
    lt.in[1] <== threshold;
    out <== lt.out;
}

template EdgeOnLine() {
    var TOLERANCE = 10000;

    signal input e1x, e1y, e2x, e2y;
    signal input lsx, lsy, lex, ley;
    signal input cs1, ca1, cs2, ca2;
    signal output out;

    component cr1 = Cross();
    cr1.p1x <== lsx; cr1.p1y <== lsy;
    cr1.p2x <== lex; cr1.p2y <== ley;
    cr1.p3x <== e1x; cr1.p3y <== e1y;
    component alt1 = AbsLessThan(64);
    alt1.val <== cr1.out;
    alt1.sign <== cs1;
    alt1.hint <== ca1;
    alt1.threshold <== TOLERANCE;
    component cr2 = Cross();
    cr2.p1x <== lsx; cr2.p1y <== lsy;
    cr2.p2x <== lex; cr2.p2y <== ley;
    cr2.p3x <== e2x; cr2.p3y <== e2y;
    component alt2 = AbsLessThan(64);
    alt2.val <== cr2.out;
    alt2.sign <== cs2;
    alt2.hint <== ca2;
    alt2.threshold <== TOLERANCE;
    out <== alt1.out * alt2.out;
}

template VerifyEdgeWithHint() {
    var MAX_POLYGONS = 4;
    var MAX_VERTICES_PER_POLYGON = 14;
    var MAX_SEGMENTS = 3;
    var FLAT_COUNT = MAX_POLYGONS * MAX_VERTICES_PER_POLYGON;

    signal input esx, esy, eex, eey;
    signal input sourceType;
    signal input sourceIndex;
    signal input edgeIndex;
    signal input polygons[4][14][2];
    signal input polygonVertexCounts[4];
    signal input segments[3][4];
    signal input cs1, ca1, cs2, ca2;
    signal output out;

    signal polySX[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal polySY[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal polyEX[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal polyEY[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal wrapDX[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal wrapDY[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];

    component isLastE[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var e = 0; e < MAX_VERTICES_PER_POLYGON; e++) {
            isLastE[p][e] = IsEqual();
            isLastE[p][e].in[0] <== e + 1;
            isLastE[p][e].in[1] <== polygonVertexCounts[p];
            polySX[p][e] <== polygons[p][e][0];
            polySY[p][e] <== polygons[p][e][1];
            var ne = (e + 1) < MAX_VERTICES_PER_POLYGON ? (e + 1) : 0;
            wrapDX[p][e] <== polygons[p][0][0] - polygons[p][ne][0];
            wrapDY[p][e] <== polygons[p][0][1] - polygons[p][ne][1];
            polyEX[p][e] <== polygons[p][ne][0] + isLastE[p][e].out * wrapDX[p][e];
            polyEY[p][e] <== polygons[p][ne][1] + isLastE[p][e].out * wrapDY[p][e];
        }
    }

    component isPoly[MAX_POLYGONS];
    component isEdge[MAX_VERTICES_PER_POLYGON];
    for (var p = 0; p < MAX_POLYGONS; p++) {
        isPoly[p] = IsEqual();
        isPoly[p].in[0] <== sourceIndex;
        isPoly[p].in[1] <== p;
    }
    for (var e = 0; e < MAX_VERTICES_PER_POLYGON; e++) {
        isEdge[e] = IsEqual();
        isEdge[e].in[0] <== edgeIndex;
        isEdge[e].in[1] <== e;
    }

    signal pSel[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal pSelSX[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal pSelSY[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal pSelEX[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    signal pSelEY[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var e = 0; e < MAX_VERTICES_PER_POLYGON; e++) {
            pSel[p][e] <== isPoly[p].out * isEdge[e].out;
            pSelSX[p][e] <== pSel[p][e] * polySX[p][e];
            pSelSY[p][e] <== pSel[p][e] * polySY[p][e];
            pSelEX[p][e] <== pSel[p][e] * polyEX[p][e];
            pSelEY[p][e] <== pSel[p][e] * polyEY[p][e];
        }
    }

    signal aSX[FLAT_COUNT + 1]; aSX[0] <== 0;
    signal aSY[FLAT_COUNT + 1]; aSY[0] <== 0;
    signal aEX[FLAT_COUNT + 1]; aEX[0] <== 0;
    signal aEY[FLAT_COUNT + 1]; aEY[0] <== 0;
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var e = 0; e < MAX_VERTICES_PER_POLYGON; e++) {
            var k = p * MAX_VERTICES_PER_POLYGON + e + 1;
            aSX[k] <== aSX[k-1] + pSelSX[p][e];
            aSY[k] <== aSY[k-1] + pSelSY[p][e];
            aEX[k] <== aEX[k-1] + pSelEX[p][e];
            aEY[k] <== aEY[k-1] + pSelEY[p][e];
        }
    }

    signal pRefSX <== aSX[FLAT_COUNT];
    signal pRefSY <== aSY[FLAT_COUNT];
    signal pRefEX <== aEX[FLAT_COUNT];
    signal pRefEY <== aEY[FLAT_COUNT];
    signal sSelSX[MAX_SEGMENTS];
    signal sSelSY[MAX_SEGMENTS];
    signal sSelEX[MAX_SEGMENTS];
    signal sSelEY[MAX_SEGMENTS];
    component isSeg[MAX_SEGMENTS];
    for (var s = 0; s < MAX_SEGMENTS; s++) {
        isSeg[s] = IsEqual();
        isSeg[s].in[0] <== sourceIndex;
        isSeg[s].in[1] <== s;
        sSelSX[s] <== isSeg[s].out * segments[s][0];
        sSelSY[s] <== isSeg[s].out * segments[s][1];
        sSelEX[s] <== isSeg[s].out * segments[s][2];
        sSelEY[s] <== isSeg[s].out * segments[s][3];
    }

    signal segRefSX <== sSelSX[0] + sSelSX[1] + sSelSX[2];
    signal segRefSY <== sSelSY[0] + sSelSY[1] + sSelSY[2];
    signal segRefEX <== sSelEX[0] + sSelEX[1] + sSelEX[2];
    signal segRefEY <== sSelEY[0] + sSelEY[1] + sSelEY[2];
    component isSrcSeg = IsEqual();
    isSrcSeg.in[0] <== sourceType;
    isSrcSeg.in[1] <== 1;

    signal refSX <== pRefSX + isSrcSeg.out * (segRefSX - pRefSX);
    signal refSY <== pRefSY + isSrcSeg.out * (segRefSY - pRefSY);
    signal refEX <== pRefEX + isSrcSeg.out * (segRefEX - pRefEX);
    signal refEY <== pRefEY + isSrcSeg.out * (segRefEY - pRefEY);
    component eol = EdgeOnLine();
    eol.e1x <== esx; eol.e1y <== esy;
    eol.e2x <== eex; eol.e2y <== eey;
    eol.lsx <== refSX; eol.lsy <== refSY;
    eol.lex <== refEX; eol.ley <== refEY;
    eol.cs1 <== cs1; eol.ca1 <== ca1;
    eol.cs2 <== cs2; eol.ca2 <== ca2;
    out <== eol.out;
}

template PointInPolygon() {
    var MAX_VERTICES_PER_PARTITION = 14;

    signal input px, py;
    signal input verts[14][2];
    signal input vc;
    signal input cSigns[14];
    signal input cAbs[14];
    signal output out;

    component edgeAct[MAX_VERTICES_PER_PARTITION];
    component isLastV[MAX_VERTICES_PER_PARTITION];
    signal nxtX[MAX_VERTICES_PER_PARTITION]; signal nxtY[MAX_VERTICES_PER_PARTITION];
    signal wDnx[MAX_VERTICES_PER_PARTITION]; signal wDny[MAX_VERTICES_PER_PARTITION];
    component cr[MAX_VERTICES_PER_PARTITION];

    signal wp[15]; signal wm[15];
    wp[0] <== 0; wm[0] <== 0;

    component viLePy[MAX_VERTICES_PER_PARTITION]; component vjGtPy[MAX_VERTICES_PER_PARTITION];
    component viGtPy[MAX_VERTICES_PER_PARTITION]; component vjLePy[MAX_VERTICES_PER_PARTITION];
    component cAbsGt[MAX_VERTICES_PER_PARTITION];

    signal cIsPos[MAX_VERTICES_PER_PARTITION]; signal cIsNeg[MAX_VERTICES_PER_PARTITION];
    signal c1a[MAX_VERTICES_PER_PARTITION]; signal c1b[MAX_VERTICES_PER_PARTITION]; signal c1c[MAX_VERTICES_PER_PARTITION];
    signal c2a[MAX_VERTICES_PER_PARTITION]; signal c2b[MAX_VERTICES_PER_PARTITION]; signal c2c[MAX_VERTICES_PER_PARTITION];
    signal csf[MAX_VERTICES_PER_PARTITION];

    for (var i = 0; i < MAX_VERTICES_PER_PARTITION; i++) {
        edgeAct[i] = LessThan(8);
        edgeAct[i].in[0] <== i;
        edgeAct[i].in[1] <== vc;
        isLastV[i] = IsEqual();
        isLastV[i].in[0] <== i + 1;
        isLastV[i].in[1] <== vc;
        var ni = (i + 1) < MAX_VERTICES_PER_PARTITION ? (i + 1) : 0;
        wDnx[i] <== verts[0][0] - verts[ni][0];
        wDny[i] <== verts[0][1] - verts[ni][1];
        nxtX[i] <== verts[ni][0] + isLastV[i].out * wDnx[i];
        nxtY[i] <== verts[ni][1] + isLastV[i].out * wDny[i];
        cr[i] = Cross();
        cr[i].p1x <== verts[i][0]; cr[i].p1y <== verts[i][1];
        cr[i].p2x <== nxtX[i];    cr[i].p2y <== nxtY[i];
        cr[i].p3x <== px;          cr[i].p3y <== py;
        cSigns[i] * (1 - cSigns[i]) === 0;
        csf[i] <== 1 - 2 * cSigns[i];
        cr[i].out === csf[i] * cAbs[i];
        viLePy[i] = LessThan(16);
        viLePy[i].in[0] <== verts[i][1];
        viLePy[i].in[1] <== py + 1;
        vjGtPy[i] = LessThan(16);
        vjGtPy[i].in[0] <== py;
        vjGtPy[i].in[1] <== nxtY[i];
        viGtPy[i] = LessThan(16);
        viGtPy[i].in[0] <== py;
        viGtPy[i].in[1] <== verts[i][1];
        vjLePy[i] = LessThan(16);
        vjLePy[i].in[0] <== nxtY[i];
        vjLePy[i].in[1] <== py + 1;
        cAbsGt[i] = LessThan(64);
        cAbsGt[i].in[0] <== 0;
        cAbsGt[i].in[1] <== cAbs[i];
        cIsPos[i] <== (1 - cSigns[i]) * cAbsGt[i].out;
        cIsNeg[i] <== cSigns[i] * cAbsGt[i].out;
        c1a[i] <== edgeAct[i].out * viLePy[i].out;
        c1b[i] <== c1a[i] * vjGtPy[i].out;
        c1c[i] <== c1b[i] * cIsPos[i];
        c2a[i] <== edgeAct[i].out * viGtPy[i].out;
        c2b[i] <== c2a[i] * vjLePy[i].out;
        c2c[i] <== c2b[i] * cIsNeg[i];
        wp[i+1] <== wp[i] + c1c[i];
        wm[i+1] <== wm[i] + c2c[i];
    }

    component wEq = IsEqual();
    wEq.in[0] <== wp[MAX_VERTICES_PER_PARTITION];
    wEq.in[1] <== wm[MAX_VERTICES_PER_PARTITION];
    out <== 1 - wEq.out;
}

template HashLevel() {
    var MAX_POLYGONS = 4;
    var MAX_VERTICES_PER_POLYGON = 14;
    var MAX_OBJECTS = 15;
    var N = MAX_POLYGONS * (MAX_VERTICES_PER_POLYGON * 2 + 1) + MAX_OBJECTS * 2 + 1;

    signal input polygons[4][14][2];
    signal input polygonVertexCounts[4];
    signal input polygonCount;
    signal input objects[15][2];
    signal input objectCount;
    signal input segmentCount;
    signal output out;

    component h[N];
    for (var i = 0; i < N; i++) {
        h[i] = Hash2();
    }

    signal chain[N + 1];
    signal en[N];
    signal df[N];
    chain[0] <== 0;

    component pAct[MAX_POLYGONS];
    for (var p = 0; p < MAX_POLYGONS; p++) {
        pAct[p] = LessThan(8);
        pAct[p].in[0] <== p;
        pAct[p].in[1] <== polygonCount;
    }

    component vAct[MAX_POLYGONS][MAX_VERTICES_PER_POLYGON];
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var v = 0; v < MAX_VERTICES_PER_POLYGON; v++) {
            vAct[p][v] = LessThan(8);
            vAct[p][v].in[0] <== v;
            vAct[p][v].in[1] <== polygonVertexCounts[p];
        }
    }

    var si = 0;
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var v = 0; v < MAX_VERTICES_PER_POLYGON; v++) {
            en[si] <== pAct[p].out * vAct[p][v].out;
            h[si].a <== chain[si]; h[si].b <== polygons[p][v][0];
            df[si] <== h[si].out - chain[si];
            chain[si+1] <== chain[si] + en[si] * df[si];
            si = si + 1;
            en[si] <== pAct[p].out * vAct[p][v].out;
            h[si].a <== chain[si]; h[si].b <== polygons[p][v][1];
            df[si] <== h[si].out - chain[si];
            chain[si+1] <== chain[si] + en[si] * df[si];
            si = si + 1;
        }
        en[si] <== pAct[p].out;
        h[si].a <== chain[si]; h[si].b <== 0xFFFFFFFF;
        df[si] <== h[si].out - chain[si];
        chain[si+1] <== chain[si] + en[si] * df[si];
        si = si + 1;
    }

    component oAct[MAX_OBJECTS];
    for (var o = 0; o < MAX_OBJECTS; o++) {
        oAct[o] = LessThan(8);
        oAct[o].in[0] <== o;
        oAct[o].in[1] <== objectCount;
    }

    for (var o = 0; o < MAX_OBJECTS; o++) {
        en[si] <== oAct[o].out;
        h[si].a <== chain[si]; h[si].b <== objects[o][0];
        df[si] <== h[si].out - chain[si];
        chain[si+1] <== chain[si] + en[si] * df[si];
        si = si + 1;
        en[si] <== oAct[o].out;
        h[si].a <== chain[si]; h[si].b <== objects[o][1];
        df[si] <== h[si].out - chain[si];
        chain[si+1] <== chain[si] + en[si] * df[si];
        si = si + 1;
    }

    en[si] <== 1;
    h[si].a <== chain[si]; h[si].b <== segmentCount;
    df[si] <== h[si].out - chain[si];
    chain[si+1] <== chain[si] + en[si] * df[si];

    out <== chain[N];
}

template Slicer() {
    var MAX_POLYGONS = 4;
    var MAX_VERTICES_PER_POLYGON = 14;
    var MAX_PARTITIONS = 20;
    var MAX_VERTICES_PER_PARTITION = 14;
    var MAX_OBJECTS = 15;
    var MAX_SEGMENTS = 3;
    var NUM_PAIRS = MAX_OBJECTS * (MAX_OBJECTS - 1) / 2;

    // Public.
    signal input levelHash;
    signal input polygonCount;
    signal input objectCount;
    signal input partitionCount;

    // Private.
    signal input polygons[4][14][2];
    signal input polygonVertexCounts[4];
    signal input objects[15][2];
    signal input segmentCount;
    signal input segments[3][4];
    signal input partitions[20][14][2];
    signal input partitionVertexCounts[20];
    signal input edgeHints[20][14][3];
    signal input edgeCrossSign1[20][14];
    signal input edgeCrossAbs1[20][14];
    signal input edgeCrossSign2[20][14];
    signal input edgeCrossAbs2[20][14];
    signal input objectHints[15];
    signal input pipCrossSigns[15][14];
    signal input pipCrossAbs[15][14];

    component hl = HashLevel();
    for (var p = 0; p < MAX_POLYGONS; p++) {
        for (var v = 0; v < MAX_VERTICES_PER_POLYGON; v++) {
            hl.polygons[p][v][0] <== polygons[p][v][0];
            hl.polygons[p][v][1] <== polygons[p][v][1];
        }
        hl.polygonVertexCounts[p] <== polygonVertexCounts[p];
    }
    hl.polygonCount <== polygonCount;
    for (var o = 0; o < MAX_OBJECTS; o++) {
        hl.objects[o][0] <== objects[o][0];
        hl.objects[o][1] <== objects[o][1];
    }
    hl.objectCount <== objectCount;
    hl.segmentCount <== segmentCount;
    levelHash === hl.out;

    component pcGe = LessThan(8);
    pcGe.in[0] <== objectCount;
    pcGe.in[1] <== partitionCount + 1;
    pcGe.out === 1;

    component partAct[MAX_PARTITIONS];
    component eAct[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    component isLastPE[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    signal nPX[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION]; signal nPY[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    signal wdpx[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION]; signal wdpy[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    component verE[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    signal eIsAct[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];
    signal eFail[MAX_PARTITIONS][MAX_VERTICES_PER_PARTITION];

    for (var p = 0; p < MAX_PARTITIONS; p++) {
        partAct[p] = LessThan(8);
        partAct[p].in[0] <== p;
        partAct[p].in[1] <== partitionCount;
    }

    for (var p = 0; p < MAX_PARTITIONS; p++) {
        for (var i = 0; i < MAX_VERTICES_PER_PARTITION; i++) {
            eAct[p][i] = LessThan(8);
            eAct[p][i].in[0] <== i;
            eAct[p][i].in[1] <== partitionVertexCounts[p];
            isLastPE[p][i] = IsEqual();
            isLastPE[p][i].in[0] <== i + 1;
            isLastPE[p][i].in[1] <== partitionVertexCounts[p];

            var ni = (i + 1) < MAX_VERTICES_PER_PARTITION ? (i + 1) : 0;
            wdpx[p][i] <== partitions[p][0][0] - partitions[p][ni][0];
            wdpy[p][i] <== partitions[p][0][1] - partitions[p][ni][1];
            nPX[p][i] <== partitions[p][ni][0] + isLastPE[p][i].out * wdpx[p][i];
            nPY[p][i] <== partitions[p][ni][1] + isLastPE[p][i].out * wdpy[p][i];
            verE[p][i] = VerifyEdgeWithHint();
            verE[p][i].esx <== partitions[p][i][0];
            verE[p][i].esy <== partitions[p][i][1];
            verE[p][i].eex <== nPX[p][i];
            verE[p][i].eey <== nPY[p][i];
            verE[p][i].sourceType <== edgeHints[p][i][0];
            verE[p][i].sourceIndex <== edgeHints[p][i][1];
            verE[p][i].edgeIndex <== edgeHints[p][i][2];

            for (var pp = 0; pp < MAX_POLYGONS; pp++) {
                for (var vv = 0; vv < MAX_VERTICES_PER_POLYGON; vv++) {
                    verE[p][i].polygons[pp][vv][0] <== polygons[pp][vv][0];
                    verE[p][i].polygons[pp][vv][1] <== polygons[pp][vv][1];
                }
                verE[p][i].polygonVertexCounts[pp] <== polygonVertexCounts[pp];
            }
            for (var s = 0; s < MAX_SEGMENTS; s++) {
                for (var c = 0; c < 4; c++) {
                    verE[p][i].segments[s][c] <== segments[s][c];
                }
            }
            verE[p][i].cs1 <== edgeCrossSign1[p][i];
            verE[p][i].ca1 <== edgeCrossAbs1[p][i];
            verE[p][i].cs2 <== edgeCrossSign2[p][i];
            verE[p][i].ca2 <== edgeCrossAbs2[p][i];
            eIsAct[p][i] <== partAct[p].out * eAct[p][i].out;
            eFail[p][i] <== eIsAct[p][i] * (1 - verE[p][i].out);
            eFail[p][i] === 0;
        }
    }

    signal pFail[MAX_OBJECTS];
    signal pidxF[MAX_OBJECTS];
    signal selVC[MAX_OBJECTS];
    signal selV[MAX_OBJECTS][MAX_VERTICES_PER_PARTITION][2];
    signal vcP[MAX_OBJECTS][MAX_PARTITIONS];
    signal aVC[MAX_OBJECTS][MAX_PARTITIONS];
    signal vxP[MAX_OBJECTS][MAX_VERTICES_PER_PARTITION][MAX_PARTITIONS];
    signal vyP[MAX_OBJECTS][MAX_VERTICES_PER_PARTITION][MAX_PARTITIONS];
    signal aVX[MAX_OBJECTS][MAX_VERTICES_PER_PARTITION][MAX_PARTITIONS];
    signal aVY[MAX_OBJECTS][MAX_VERTICES_PER_PARTITION][MAX_PARTITIONS];

    component objAct[MAX_OBJECTS];
    component clValid[MAX_OBJECTS];
    component pip[MAX_OBJECTS];
    component isPart[MAX_OBJECTS][MAX_PARTITIONS];
    for (var o = 0; o < MAX_OBJECTS; o++) {
        objAct[o] = LessThan(8);
        objAct[o].in[0] <== o;
        objAct[o].in[1] <== objectCount;

        clValid[o] = LessThan(8);
        clValid[o].in[0] <== objectHints[o];
        clValid[o].in[1] <== partitionCount;

        pidxF[o] <== objAct[o].out * (1 - clValid[o].out);
        pidxF[o] === 0;

        for (var pp = 0; pp < MAX_PARTITIONS; pp++) {
            isPart[o][pp] = IsEqual();
            isPart[o][pp].in[0] <== objectHints[o];
            isPart[o][pp].in[1] <== pp;
        }

        for (var pp = 0; pp < MAX_PARTITIONS; pp++) {
            vcP[o][pp] <== isPart[o][pp].out * partitionVertexCounts[pp];
        }
        aVC[o][0] <== vcP[o][0];
        for (var pp = 1; pp < MAX_PARTITIONS; pp++) {
            aVC[o][pp] <== aVC[o][pp-1] + vcP[o][pp];
        }
        selVC[o] <== aVC[o][MAX_PARTITIONS - 1];

        for (var v = 0; v < MAX_VERTICES_PER_PARTITION; v++) {
            for (var pp = 0; pp < MAX_PARTITIONS; pp++) {
                vxP[o][v][pp] <== isPart[o][pp].out * partitions[pp][v][0];
                vyP[o][v][pp] <== isPart[o][pp].out * partitions[pp][v][1];
            }
            aVX[o][v][0] <== vxP[o][v][0];
            aVY[o][v][0] <== vyP[o][v][0];
            for (var pp = 1; pp < MAX_PARTITIONS; pp++) {
                aVX[o][v][pp] <== aVX[o][v][pp-1] + vxP[o][v][pp];
                aVY[o][v][pp] <== aVY[o][v][pp-1] + vyP[o][v][pp];
            }
            selV[o][v][0] <== aVX[o][v][MAX_PARTITIONS - 1];
            selV[o][v][1] <== aVY[o][v][MAX_PARTITIONS - 1];
        }

        pip[o] = PointInPolygon();
        pip[o].px <== objects[o][0];
        pip[o].py <== objects[o][1];
        for (var v = 0; v < MAX_VERTICES_PER_PARTITION; v++) {
            pip[o].verts[v][0] <== selV[o][v][0];
            pip[o].verts[v][1] <== selV[o][v][1];
        }
        pip[o].vc <== selVC[o];
        for (var v = 0; v < MAX_VERTICES_PER_PARTITION; v++) {
            pip[o].cSigns[v] <== pipCrossSigns[o][v];
            pip[o].cAbs[v] <== pipCrossAbs[o][v];
        }

        pFail[o] <== objAct[o].out * (1 - pip[o].out);
        pFail[o] === 0;
    }

    component pairEq[NUM_PAIRS];
    signal bAct[NUM_PAIRS];
    signal uFail[NUM_PAIRS];
    var pi = 0;
    for (var i = 0; i < MAX_OBJECTS; i++) {
        for (var j = i + 1; j < MAX_OBJECTS; j++) {
            pairEq[pi] = IsEqual();
            pairEq[pi].in[0] <== objectHints[i];
            pairEq[pi].in[1] <== objectHints[j];
            bAct[pi] <== objAct[i].out * objAct[j].out;
            uFail[pi] <== bAct[pi] * pairEq[pi].out;
            uFail[pi] === 0;
            pi = pi + 1;
        }
    }
}

component main {public [levelHash, polygonCount, objectCount, partitionCount]} = Slicer();
