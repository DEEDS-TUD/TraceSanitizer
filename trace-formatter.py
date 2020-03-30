#!/usr/bin/env python3
from os import listdir
from os import rename
from os import mkdir
from shutil import copy
from shutil import rmtree

import trformatter
import logging
import fileinput
import argparse
import re

header = ['Timestamp', 'TID', 'IID', 'OPName', 'Value']
ignore = ["GlobalVariables", "Mapping", '#TraceStartInstNumber', 'FAULT']
include = [',load,', ',store,', ',br,',',call-', ',alloca,', ',ret-']

load_entry = '(load(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){2})'
br2_entry = '(br(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){2})'
br4_entry = '(br(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){4})'
store_entry = '(store(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){3})'
alloca_entry = '(alloca(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){3})'
ret_entry = '(ret-[\w.]+(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+){1,2})'
call_entry = '(call-[\w.]+(-\w),(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+(,(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+)*)'
#call_entry = 'call-[\w.]+(-\w),(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+(,((0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+)*)*'
entries = [load_entry, br2_entry, br4_entry, store_entry, alloca_entry, ret_entry, call_entry]

entry =  '[0-9]+,[0-9]+,[0-9]+,(' + '|'.join(entries) +  ')\n'
#entry = '[0-9]+,[0-9]+,[0-9]+,call-[\w.]+(-\w),(0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+(,((0[0-9]|1[0-6])-[0-9]+-[0-9a-f]+)*)*\n'

reg = re.compile(entry)

def is_ignore(line):
    for i in ignore:
        if line.startswith(i):
            return True
    return False

def is_include(line):
    return reg.fullmatch(line)

def getNumOperandsAndHeal(f, folder):
    logging.info('Dealing with ' + f)
    numOp = 0
    towrite = ''
    with open(folder + '/' + f, 'r', errors='replace') as data: 
        for line in data:
            if is_ignore(line):
                towrite += line
                continue
            if not is_include(line):
                for i in include:
                    if i in line:
                        logging.warn("Issue with line: " + line)
                #logging.warn("Problem with file " + folder + '/' + f + ' at line ' + line)
                continue
            towrite += line
#            if len(line.split(';')) <= 3:
#                continue
            numOp = max(numOp, len(line.split(',')) - 5)
    with open(folder + '/' +f, 'w') as dst:
        dst.write(towrite)

    return numOp

def updateHeader(numOp):
    logging.info("Updating the headers...")
    for i in range(numOp):
        header.append('Operand-' + str(i))

def writeHeaderandPadding(folder, f):
    logging.info('Padding of ' + f)
    data = []
    with open(folder + '/' + f, 'r') as srcfile:
        data = srcfile.readlines()
    res = ''
    for line in data:
        if is_ignore(line):
            continue
        padding = [''] * (len(header) - len(line.split(',')))
        tmp = line
        if len(padding) > 0:
            tmp = tmp.strip() + "," + ','.join(padding) + '\n'
        res += tmp
    with open(folder + '/' + f, 'w') as tmpfile:
        tmpfile.write(','.join(header) + '\n')
        tmpfile.write(res)
        #for line in data:
        #    if is_ignore(line):
        #        continue
        #    padding = [''] * (len(header) - len(line.split(',')))
        #    tmp = line
        #    if len(padding) > 0:
        #        tmp = tmp.strip() + "," + ','.join(padding) + '\n'
        #    tmpfile.write(tmp)
#    rename(folder + '/' + f + '.tmp', folder + '/' + f)

def merge_globals_mapping_faults(folder, src):
    files = [f for f in listdir(src) if f == 'globals']
    res = ''
    with open(folder + '/trace_linear/globals', 'a+') as dst:
        for fl in files:
            with open(src + '/' + fl, 'r') as data:
                res += ''.join(data.readlines())
        dst.write(res)

    files = [f for f in listdir(src) if f == 'mapping']
    res = ''
    with open(folder + '/trace_linear/mapping', 'a+') as dst:
        for fl in files:
            with open(src + '/' + fl, 'r') as data:
                tmp = data.readlines()
                res += ''.join(tmp)
        dst.write(res)
    files = [f for f in listdir(src) if f == 'faultinj']
    res = ''
    with open(folder + '/trace_linear/faultinj', 'a+') as dst:
        for fl in files:
            with open(src + '/' + fl, 'r') as data:
                tmp = data.readlines()
                res += ''.join(tmp)
        dst.write(res)

    

def main(folder):
    files = [f for f in listdir(folder) if "trace" in f and not f.endswith('tmp')]
    prob = []
    try:
        numOp = 0
        for f in files:
            numOp = max(numOp, getNumOperandsAndHeal(f, folder))
        logging.info('maximal number of operands: ' + str(numOp))
        updateHeader(numOp)
        mkdir(folder + '/trace_linear')
        for f in files:
            outdir = folder + '/' + f.replace('llfi.stat.', '').replace('.txt', '')
            mkdir(outdir)
            trformatter.main(folder, outdir, f)
            writeHeaderandPadding(outdir, f.replace('.txt', ''))
            merge_globals_mapping_faults(folder, outdir)
        logical_mapping = ""
        for f in files:
            logical_mapping += f.replace('llfi.stat.trace', '').replace('.txt', '').replace('-', ',') +'\n'

        once = True
        for f in files:    
            outdir = folder + '/' + f.replace('llfi.stat.', '').replace('.txt', '')
            with open(outdir + '/logical_mapping', 'w') as dst:
                dst.write(logical_mapping)
            copy(folder + '/trace_linear/globals', outdir + '/')
            copy(folder + '/trace_linear/mapping', outdir + '/')
            copy(folder + '/trace_linear/faultinj', outdir + '/')
            if once:
                copy(outdir + "/logical_mapping", folder + '/trace_linear/')
                once = False
    except:
        prob.append(folder)
        print('removing ' + folder)
        rmtree(folder)
    if len(prob) > 0:
        print('There were problem with the following folders:')
    for p in prob:
        print(p)
logging.basicConfig(level=logging.WARN, format='%(asctime)s %(message)s', datefmt='%m/%d/%Y %I:%M:%S%p')
parser = argparse.ArgumentParser()
parser.add_argument("directory", help="the path to the traces")
args = parser.parse_args()
main(args.directory)
