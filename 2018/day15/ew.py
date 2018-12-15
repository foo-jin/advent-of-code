#!/usr/bin/env python
from collections import deque, Counter
from itertools import count, product
with open('input.txt', 'r') as ifile:
    field = [list(row.strip().split()[0]) for row in ifile]
fldht, fldwh = len(field), len(field[0])
units = []
for y, x in product(range(fldht), range(fldwh)):
    if field[y][x] in 'GE':
        units.append([200, y, x, field[y][x]])
        field[y][x] = len(units) - 1
elves = Counter(unit[3] for unit in units)['E']

def find_mov_target(field, unit):
    start = (unit[1], unit[2])
    queue = deque([start])
    cseen = {}
    while True:
        try:
            cnode = queue.pop()
        except IndexError: # Runs out of spaces to move
            return None
        for dy, dx in ((-1, 0), (0, -1), (0, 1), (1, 0)):
            j, i = cnode[0] + dy, cnode[1] + dx
            if 0 <= j < fldht and 0 <= i < fldwh:
                nnode = (j, i)
                cell = field[j][i]
                if isinstance(cell, list) and cell[3] != unit[3]:
                    if cnode == start: # Enemy is right in front of it
                        return None
                    nnode = cnode
                    while cseen[nnode] != start:
                        nnode = cseen[nnode]
                    return nnode
                if cell == '.' and nnode not in cseen:
                    queue.appendleft(nnode)
                    cseen[nnode] = cnode

def find_atk_target(field, unit):
    tlist = []
    for dy, dx in ((-1, 0), (0, -1), (0, 1), (1, 0)):
        j, i = unit[1] + dy, unit[2] + dx
        if 0 <= j < fldht and 0 <= i < fldwh:
            cell = field[j][i]
            if isinstance(cell, list) and cell[3] != unit[3]:
                tlist.append(cell)
    if tlist:
        return min(tlist, key=lambda i: i[0])
    return None

def sim_battle(field, units, elfpw):
    for turn in count():
        units = sorted(units, key=lambda i: tuple(i[1:3]))
        utlen = len(units)
        for uidx, unit in enumerate(units):
            if unit[0] < 1: # Dead Elves/Goblins don't fight
                continue
            hdcnt = Counter(unit[3] for unit in units if unit[0] > 0)
            if hdcnt['G'] == 0 or hdcnt['E'] == 0:
                print('turn: {}', turn)
                return hdcnt, turn * sum(unit[0] for unit in units if unit[0] > 0)
            trgt = find_mov_target(field, unit)
            if trgt: # Movement step
                field[unit[1]][unit[2]] = '.'
                unit[1:3] = trgt
                field[unit[1]][unit[2]] = unit
            trgt = find_atk_target(field, unit)
            if trgt: # Attack step
                trgt[0] -= elfpw if unit[3] == 'E' else 3 
                if trgt[0] < 1:
                    field[trgt[1]][trgt[2]] = '.'
        units = [unit for unit in units if unit[0] > 0]

for elfpw in count(3):
    utcpy = [unit[:] for unit in units]
    fdcpy = []
    for row in field:
        fdcpy.append([])
        for cell in row:
            if isinstance(cell, str):
                fdcpy[-1].append(cell)
            else:
                fdcpy[-1].append(utcpy[cell])
    btout = sim_battle(fdcpy, utcpy, elfpw)
    if elfpw == 3:
        print(btout[1]) # 1
    if btout[0]['E'] == elves:
        print(btout[1]) # 2
        break
