import {readFileSync, writeFileSync} from 'fs';
import { exec } from 'child_process';

const isNumeric = (value) => {
  return /^\d+$/.test(value);
}

const getBranch = () => new Promise((resolve, reject) => {
    return exec('git branch --show-current', (err, stdout, stderr) => {
        if (err)
            reject(`getBranch Error: ${err}`);
        else if (typeof stdout === 'string')
            resolve(stdout.trim());
    });
});

const getGitRoot = () => new Promise((resolve, reject) => {
  return exec('git rev-parse --show-toplevel', (err, stdout, stderr) => {
      if (err)
          reject(`getBranch Error: ${err}`);
      else if (typeof stdout === 'string')
          resolve(stdout.trim());
  });
});

const updateCommitMsg = async () => {
  
  const messageFilePath = (await getGitRoot()) + '/.git/COMMIT_EDITMSG';
  const message = readFileSync(messageFilePath).toString();
  
  const branchName = await getBranch();
  const indexOfTaskID = branchName.indexOf('cod-');
  const taskID = branchName.substring(indexOfTaskID).split('-')[1];

  if (isNumeric(taskID) && message) {
    const prependText = `COD-${taskID}:`;
    if (message.substring(0, prependText.length) === prependText) {
      return;
    }

    writeFileSync(messageFilePath, `${prependText} ${message}`);
  }

}


updateCommitMsg();
